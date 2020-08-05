/*
Copyright (C) 2020 OAX Foundation Limited

This program is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software Foundation,
either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <http://www.gnu.org/licenses/>.
*/

#![cfg_attr(not(feature = "std"), no_std)]
//! #PRC20 Module
//! A runtime module for an ERC20 equivalent token standard
//! + a few extra handy features!
//! Additional Features:
//! 1) Atomic Swap :
//!     Allows swapping tokens with another user in a single tx
//! 2) Multi-transfer:
//! Allows transferring tokens to multiple users, in one single tx
use codec::{Codec, Decode, Encode};
use frame_support::traits::Get;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, Parameter,
};
use frame_system::{self as system, ensure_signed};
use sp_arithmetic::traits::BaseArithmetic;
use sp_runtime::traits::{
    CheckedAdd, CheckedSub, IdentifyAccount, Member, One, StaticLookup, Verify,
};
use sp_std::{convert::TryInto, vec::Vec};

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + pallet_balances::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Custom types for tokens
    type TokenBalance: Parameter + Member + Codec + Default + Copy + BaseArithmetic;
    type TokenId: Parameter + Member + Codec + Default + Copy + BaseArithmetic;
    /// Additional types for atomic swap
    type Public: IdentifyAccount<AccountId = Self::AccountId>;
    type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
    type MaxTransfers: Get<u8>;
}

/// Offer struct used in atomic swaps, this is not signed
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct Offer<TokenBalance, TokenId> {
    pub offer_token: TokenId,
    pub offer_amount: TokenBalance,
    pub requested_token: TokenId,
    pub requested_amount: TokenBalance,
    pub nonce: u128,
}

/// Signed version of the offer struct, used in atomic swaps
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct SignedOffer<Signature, AccountId, TokenBalance, TokenId> {
    pub offer: Offer<TokenBalance, TokenId>,
    pub signature: Signature,
    pub signer: AccountId,
}

/// struct used for Multi Transfers
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct TokenTransferDetails<AccountId, TokenBalance> {
    pub amount: TokenBalance,
    pub to: AccountId,
}

decl_error! {
    pub enum Error for Module<T: Trait>{
        /// too many prc20 tokens in chain, limit for TokenCount
        MaxTokenLimitReached,
        /// got underflow while subtracting (used in TransferFrom)
        UnderFlow,
        /// balance too low to send value
        InsufficientBalance,
        /// Wrong Offline Signature
        InvalidSignature,
        /// Wrong Nonce
        IncorrectNonce,
        /// too many multiTransfers
        /// based on the MaxTransfers u32 set in lib.rs
        LimitExceeded,
    }
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as PRC20 {
        /// storage maps and value needed for operating a token
        /// this stores the total supply of each token
        TotalSupply get(fn total_supply): map hasher(blake2_128_concat)
            T::TokenId => T::TokenBalance;
        /// this stores balance maps for each token + addr
        Balances get(fn balance_of): map hasher(blake2_128_concat)
            (T::TokenId, T::AccountId) => T::TokenBalance;
        /// this stores allowances (to enable transferFrom)
        Allowance get(fn allowance_of): map hasher(blake2_128_concat)
            (T::TokenId, T::AccountId, T::AccountId) => T::TokenBalance;
        /// this stores the total number of different tokens in the blockchain
        TokenCount get(fn token_count): T::TokenId;
    }
}

// The module's dispatch functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events

        /// this is needed only if you are using events in your module
        fn deposit_event() = default;

        ///create a new token,
        /// passing totalSupply, (currently creator will receive total supply)
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 70_000_000]
        fn create_token(origin,
            #[compact] total_supply: T::TokenBalance
        ) -> DispatchResult {
            // ensure signed from the sender
            let sender = ensure_signed(origin)?;
            // count the current token id
            let current_id = Self::token_count();
            // add one to the id using checked_add, if it errors out broadcast
            // a MaxTokenLimitReached error
            let next_id = match current_id.checked_add(&One::one()) {
                Some(r) => r,
                None => return Err(<Error<T>>::MaxTokenLimitReached.into()),
            };
            // Update the token count
            <TokenCount<T>>::put(next_id);
            // Add the currency id and total supply
            <TotalSupply<T>>::insert(current_id, total_supply);
            // in this example we send the total supply to the creator
            <Balances<T>>::insert((current_id, sender.clone()), total_supply);
            // Broadcast a NewToken event
            Self::deposit_event(
                RawEvent::NewToken(current_id, sender, total_supply));
            Ok(())
        }

        /// do transfers like erc20 ( TokenId, To, Amount)
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 70_000_000]
        fn transfer(origin,
            to: <T::Lookup as StaticLookup>::Source,
            #[compact] id: T::TokenId,
            #[compact] amount: T::TokenBalance
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            // convert from lookup to T::AccountId
            let to = T::Lookup::lookup(to)?;
            // do balance check
            let enough_balance = Self::check_enough_balance(id,
                sender.clone(),
                amount);
            ensure!(enough_balance, <Error<T>>::InsufficientBalance);
            // make the transfer
            Self::make_transfer(id, sender, to, amount)
        }

        /// do approval like erc20 (TokenId, To, Amount)
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 70_000_000]
        fn approve(origin,
            spender: <T::Lookup as StaticLookup>::Source,
            #[compact] id: T::TokenId,
            #[compact] value: T::TokenBalance
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            // convert from lookup to T::AccountId
            let spender = T::Lookup::lookup(spender)?;
            // add to allowance
            //(here we don't mind someone setting 0 allowance)
            <Allowance<T>>::insert((id,
                sender.clone(),
                spender.clone()),
                value);
            // broadcast a Approval Event
            Self::deposit_event(
                RawEvent::Approval(id, sender, spender, value));
            Ok(())
        }

        /// do transfer from
        ///(allows approver to spend token)(TokenId, From, To, Amount)
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 70_000_000]
        fn transfer_from(origin,
            from: <T::Lookup as StaticLookup>::Source,
            to: <T::Lookup as StaticLookup>::Source,
            #[compact] id: T::TokenId,
            #[compact] value: T::TokenBalance
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            // convert from lookup to T::AccountId
            let from = T::Lookup::lookup(from)?;
            let to = T::Lookup::lookup(to)?;
            // check allowance
            let allowance = Self::allowance_of((id,
                from.clone(),
                sender.clone()));
            // check new allowance if transfer is made,
            // if there is an underflow, error with underflow error
            let updated_allowance = match allowance.checked_sub(&value) {
                Some(r) => r,
                None => return Err(<Error<T>>::UnderFlow.into()),
            };
            let enough_balance = Self::check_enough_balance(id,
                from.clone(),
                value);
            ensure!(enough_balance, <Error<T>>::InsufficientBalance);
            // make the transfer
            Self::make_transfer(id, from.clone(), to.clone(), value)?;
            // update the allowance
            <Allowance<T>>::insert((id, from, sender), updated_allowance);
            Ok(())
        }

        /// atomic swap functionality for tokens
        /// note that 0 token amounts are accepted
        /// since this allows a user to offer someone his tokens for free
        /// without paying any network fees
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 70_000_000]
        fn swap(origin,
            signed_offer:
            SignedOffer<T::Signature,T::AccountId, T::TokenBalance, T::TokenId>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            // Ensure that the SignedOffer is signed correctly
            ensure!(Self::verify_offer_signature(signed_offer.clone()).is_ok(),
                <Error<T>>::InvalidSignature);
            // Make the Swap
            Self::make_swap(sender, signed_offer)
        }

        /// multi transfer functionality for tokens
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 70_000_000]
        fn multi_transfer(origin,
            #[compact] id: T::TokenId,
            td_vec: Vec<TokenTransferDetails<T::AccountId, T::TokenBalance>>
        ) -> DispatchResult{
            // check if signed
            let sender = ensure_signed(origin)?;
            // get total number of transfer details
            let num_transfers = td_vec.len();
            //limit this to a certain amount of multiTransfers
            ensure!((num_transfers as u32) < (T::MaxTransfers::get() as u32),
                <Error<T>>::LimitExceeded);
            // build a status vector to push status of each transfer
            let mut status_vector: Vec<(T::AccountId, T::TokenBalance, bool)>=
                Vec::new();
            // iterate
            for i in 0..num_transfers{
                // Ensure enough balance (returns true or false)
                //(if false we just don't make the transfer,
                // and set the status to false in our event)
                let status = Self::check_enough_balance(id,
                    sender.clone(),
                    td_vec[i].amount.clone());
                // if enough balance make the transfer
                if status{
                    // should not fail, so we don't care about the return
                    Self::make_transfer(id,
                        sender.clone(),
                        td_vec[i].to.clone(),
                        td_vec[i].amount.clone())?;
                }
                // push to status vector
                status_vector.push((td_vec[i].to.clone(),
                    td_vec[i].amount,
                    status));
            }
            // broadcast multi transfer event
            Self::deposit_event(RawEvent::MultiTransfer(sender, status_vector));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        TokenId = <T as Trait>::TokenId,
        TokenBalance = <T as Trait>::TokenBalance,
    {
        /// New token creation (tokenId, Creator AccountId, Amount)
        NewToken(TokenId, AccountId, TokenBalance),
        /// Simple token transfer (tokenId, Sender AccountId, Recipient AccountId, Amount)
        Transfer(TokenId, AccountId, AccountId, TokenBalance),
        /// Approval (tokenId, Sender AccountId, Spender AccountId, Amount)
        Approval(TokenId, AccountId, AccountId, TokenBalance),
        /// Token Swap (offerTokenId, offerAmount, requestedTokenId, requestedAmount, maker, taker)
        Swap(
            TokenId,
            TokenBalance,
            TokenId,
            TokenBalance,
            AccountId,
            AccountId,
        ),
        /// MultiTransfer (Vec<(Destination, Amount, Successful )>)
        MultiTransfer(AccountId, Vec<(AccountId, TokenBalance, bool)>),
    }
);

impl<T: Trait> Module<T> {
    fn check_enough_balance(id: T::TokenId, from: T::AccountId, amount: T::TokenBalance) -> bool {
        let from_balance = Self::balance_of((id, from.clone()));
        from_balance >= amount
    }

    ///transfer
    fn make_transfer(
        id: T::TokenId,
        from: T::AccountId,
        to: T::AccountId,
        amount: T::TokenBalance,
    ) -> DispatchResult {
        // // get balance of account
        let from_balance = Self::balance_of((id, from.clone()));
        // modify sender and receiver balance map
        // Reduce sender balance to (from_balance - amount)
        <Balances<T>>::insert((id, from.clone()), from_balance - amount);
        // Increase receiver balance to (balance + amount)
        <Balances<T>>::mutate((id, to.clone()), |balance| *balance += amount);
        // broadcast a transfer event
        Self::deposit_event(RawEvent::Transfer(id, from.clone(), to, amount));
        Ok(())
    }

    ///swap
    fn make_swap(
        sender: T::AccountId,
        signed_offer: SignedOffer<T::Signature, T::AccountId, T::TokenBalance, T::TokenId>,
    ) -> DispatchResult {
        // Check from balance of offer creator
        let offer_from_balance =
            Self::balance_of((signed_offer.offer.offer_token, signed_offer.signer.clone()));
        // ensure offerer has enough tokens or error
        ensure!(
            offer_from_balance >= signed_offer.offer.offer_amount,
            <Error<T>>::InsufficientBalance
        );
        // Check from balance of requestor
        let requested_from_balance =
            Self::balance_of((signed_offer.offer.requested_token, sender.clone()));
        // ensure requestor has enough tokens or error
        ensure!(
            requested_from_balance >= signed_offer.offer.requested_amount,
            <Error<T>>::InsufficientBalance
        );
        // get maker nonce
        let maker_nonce: u128 =
            TryInto::<u128>::try_into(<system::Module<T>>::account_nonce(&signed_offer.signer))
                .map_err(|_| "error")?;
        // ensure maker nonce is correct (replay protection) or error
        ensure!(
            maker_nonce == signed_offer.offer.nonce,
            <Error<T>>::IncorrectNonce
        );
        // modify sender and receiver balance map
        // subtract offer token from maker
        <Balances<T>>::insert(
            (signed_offer.offer.offer_token, signed_offer.signer.clone()),
            offer_from_balance - signed_offer.offer.offer_amount,
        );
        // add offer token to taker
        <Balances<T>>::mutate(
            (signed_offer.offer.offer_token, sender.clone()),
            |balance| *balance += signed_offer.offer.offer_amount,
        );
        // subtract requested token from taker
        <Balances<T>>::insert(
            (signed_offer.offer.requested_token, sender.clone()),
            requested_from_balance - signed_offer.offer.requested_amount,
        );
        // add requested token to maker
        <Balances<T>>::mutate(
            (
                signed_offer.offer.requested_token,
                signed_offer.signer.clone(),
            ),
            |balance| *balance += signed_offer.offer.requested_amount,
        );
        // increment account nonce for replay protection
        <system::Module<T>>::inc_account_nonce(&signed_offer.signer);
        // broadcast deposit event
        Self::deposit_event(RawEvent::Swap(
            signed_offer.offer.offer_token,
            signed_offer.offer.offer_amount,
            signed_offer.offer.requested_token,
            signed_offer.offer.requested_amount,
            signed_offer.signer,
            sender,
        ));

        Ok(())
    }

    /// Verifies that the signed offer is signed by the correct signer
    fn verify_offer_signature(
        signed_offer: SignedOffer<T::Signature, T::AccountId, T::TokenBalance, T::TokenId>,
    ) -> Result<(), &'static str> {
        match signed_offer
            .signature
            .verify(&signed_offer.offer.encode()[..], &signed_offer.signer)
        {
            true => Ok(()),
            false => Err("signature is invalid"),
        }
    }
}

// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_noop, assert_ok, impl_outer_event, impl_outer_origin, parameter_types,
        weights::Weight,
    };
    use sp_core::sr25519;
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };
    use substrate_test_client::{self, AccountKeyring};

    mod prc20 {
        pub use super::super::*;
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    impl_outer_event! {
        pub enum Event for Test {
            system<T>,
            pallet_balances<T>,
            prc20<T>,
        }
    }

    /// The signature type used by accounts/transactions.
    pub type Signature = sr25519::Signature;
    /// An identifier for an account on this system.
    pub type AccountId = <Signature as Verify>::Signer;
    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of
    // the configuration traits of modules we want to use.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Test;

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }
    impl frame_system::Trait for Test {
        type Origin = Origin;
        type BaseCallFilter = ();
        type Index = u64;
        type BlockNumber = u64;
        type Call = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = Event;
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type DbWeight = ();
        type BlockExecutionWeight = ();
        type ExtrinsicBaseWeight = ();
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type Version = ();
        type ModuleToIndex = ();
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
    }

    parameter_types! {
        pub const MaxTransfers: u8 = 100;
    }
    impl Trait for Test {
        type Event = Event;
        type TokenBalance = u128;
        type TokenId = u128;
        type Public = AccountId;
        type Signature = Signature;
        type MaxTransfers = MaxTransfers;
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
    }
    impl pallet_balances::Trait for Test {
        type Balance = u64;
        type Event = Event;
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type DustRemoval = ();
    }

    pub struct ExtBuilder;
    type System = frame_system::Module<Test>;
    // type Balances = pallet_balances::Module<Test>;
    type PRC20 = Module<Test>;

    impl ExtBuilder {
        pub fn build() -> sp_io::TestExternalities {
            let mut t = system::GenesisConfig::default()
                .build_storage::<Test>()
                .unwrap();
            pallet_balances::GenesisConfig::<Test> { balances: vec![] }
                .assimilate_storage(&mut t)
                .unwrap();
            t.into()
        }
    }

    #[test]
    fn initial_token_count_is_zero() {
        ExtBuilder::build().execute_with(|| {
            // Make sure token count is 0
            assert_eq!(PRC20::token_count(), 0);
        });
    }

    #[test]
    fn create_token_works() {
        ExtBuilder::build().execute_with(|| {
            let alice = AccountId::from(AccountKeyring::Alice);
            // Create Token
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // Make sure token count is now 1
            assert_eq!(PRC20::token_count(), 1);
            // Make sure the creator has 10000 tokens
            assert_eq!(PRC20::balance_of((0, alice)), 10000);
        });
    }

    #[test]
    fn transfer_token_works() {
        ExtBuilder::build().execute_with(|| {
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            // Create Token
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // the above create_token deposits the newly created tokens
            // into alice's account ensure that the create_token succeeded by
            // checking Alice's balance for 10000
            assert_eq!(PRC20::balance_of((0, alice.clone())), 10000);
            // Make sure reciever has 0 tokens
            assert_eq!(PRC20::balance_of((0, bob.clone())), 0);
            // make sure transfer goes ok and transfer
            assert_ok!(PRC20::transfer(Origin::signed(alice), bob.clone(), 0, 10));
            // make sure reciever balance is 10
            assert_eq!(PRC20::balance_of((0, bob)), 10);
        });
    }

    #[test]
    fn transfer_fails_with_no_balance() {
        ExtBuilder::build().execute_with(|| {
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            // Create Token
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // Make sure the creator has 10000 tokens
            assert_eq!(PRC20::balance_of((0, alice.clone())), 10000);
            // should fail
            assert_noop!(
                PRC20::transfer(Origin::signed(bob), alice, 0, 10),
                Error::<Test>::InsufficientBalance
            );
        });
    }

    #[test]
    fn approve_token_works() {
        ExtBuilder::build().execute_with(|| {
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            // Make sure token count is 0
            assert_eq!(PRC20::token_count(), 0);
            // Create Token
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // Make sure token count is now 1
            assert_eq!(PRC20::token_count(), 1);
            // Make sure the creator has 10000 tokens
            assert_eq!(PRC20::balance_of((0, alice.clone())), 10000);

            // Maybe make sure initial approval is 0
            assert_eq!(PRC20::allowance_of((0, alice.clone(), bob.clone())), 0);
            // Approve account 1 to use 10 tokens from account 0
            assert_ok!(PRC20::approve(
                Origin::signed(alice.clone()),
                bob.clone(),
                0,
                10
            ));
            // Maybe make sure current approval is 10
            assert_eq!(PRC20::allowance_of((0, alice, bob)), 10);
        });
    }

    #[test]
    fn transfer_from_works() {
        ExtBuilder::build().execute_with(|| {
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            // Make sure token count is 0
            assert_eq!(PRC20::token_count(), 0);
            // Create Token
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // Make sure token count is now 1
            assert_eq!(PRC20::token_count(), 1);
            // Make sure the creator has 10000 tokens
            assert_eq!(PRC20::balance_of((0, alice.clone())), 10000);
            // Maybe make sure initial approval is 0
            assert_eq!(PRC20::allowance_of((0, alice.clone(), bob.clone())), 0);
            // Approve account 1 to use 10 tokens from account 0
            assert_ok!(PRC20::approve(
                Origin::signed(alice.clone()),
                bob.clone(),
                0,
                10
            ));
            // Maybe make sure current approval is 10
            assert_eq!(PRC20::allowance_of((0, alice.clone(), bob.clone())), 10);
            // Now lets use transfer_from
            assert_ok!(PRC20::transfer_from(
                Origin::signed(bob.clone()),
                alice,
                bob.clone(),
                0,
                10
            ));
            // Make sure balance is updated correctly
            assert_eq!(PRC20::balance_of((0, bob)), 10);
        });
    }

    #[test]
    fn swap_works() {
        ExtBuilder::build().execute_with(|| {
            // get account id for alice and bob
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            // get account keyring for Bob
            let bob_keyring = AccountKeyring::Bob;

            // Alice creates token 0
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // Bob creates token 1
            assert_ok!(PRC20::create_token(Origin::signed(bob.clone()), 10000));

            // Now bob creates an offer struct
            let offer = Offer {
                offer_token: 1,
                offer_amount: 100,
                requested_token: 0,
                requested_amount: 50,
                nonce: 0, // nonce is 0 cuz in tests, nonce doesn't increment
            };
            // bob signs this using bob_keyring to create a signed_offer
            let signed_offer = SignedOffer {
                offer: offer.clone(),
                signer: bob.clone(),
                signature: Signature::from(bob_keyring.sign(&offer.encode())),
            };
            // make sure swap is ok
            assert_ok!(PRC20::swap(Origin::signed(alice.clone()), signed_offer));
            // Bob has 50 token 0
            assert_eq!(PRC20::balance_of((0, bob.clone())), 50);
            // Bob has 9900 token 1
            assert_eq!(PRC20::balance_of((1, bob)), 9900);
            // Alice has 9950 token 0
            assert_eq!(PRC20::balance_of((0, alice.clone())), 9950);
            // Alice has 100 token 1
            assert_eq!(PRC20::balance_of((1, alice)), 100);
            // TODO: check bob nonce has increased
            // assert_eq!( Test::check_nonce(&bob), 2);
        });
    }

    #[test]
    fn swap_fails_not_enough_balance() {
        ExtBuilder::build().execute_with(|| {
            // get account id for alice and bob
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            // get account keyring for Bob
            let bob_keyring = AccountKeyring::Bob;
            // Alice creates token 0
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // Bob creates token 1
            assert_ok!(PRC20::create_token(Origin::signed(bob.clone()), 10000));

            // Now bob creates an offer struct
            // (invalid since Bob owns token 1 and not 0)
            let offer = Offer {
                offer_token: 0,
                offer_amount: 100,
                requested_token: 1,
                requested_amount: 50,
                nonce: 0,
            };
            // bob signs this using bob_keyring to create a signed_offer
            let signed_offer = SignedOffer {
                offer: offer.clone(),
                signer: bob.clone(),
                signature: Signature::from(bob_keyring.sign(&offer.encode())),
            };
            // make sure swap fails
            assert_noop!(
                PRC20::swap(Origin::signed(alice.clone()), signed_offer),
                Error::<Test>::InsufficientBalance
            );
            // Bob has 0 token 0
            assert_eq!(PRC20::balance_of((0, bob.clone())), 0);
            // Bob has 9900 token 1
            assert_eq!(PRC20::balance_of((1, bob)), 10000);
            // Alice has 9950 token 0
            assert_eq!(PRC20::balance_of((0, alice.clone())), 10000);
            // Alice has 100 token 1
            assert_eq!(PRC20::balance_of((1, alice)), 0);
        });
    }

    #[test]
    fn swap_fails_wrong_nonce() {
        ExtBuilder::build().execute_with(|| {
            // get account id for alice and bob
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            // get account keyring for Bob
            let bob_keyring = AccountKeyring::Bob;
            // Alice creates token 0
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // Bob creates token 1
            assert_ok!(PRC20::create_token(Origin::signed(bob.clone()), 10000));
            // Now bob creates an offer struct with wrong nonce
            let offer = Offer {
                offer_token: 1,
                offer_amount: 100,
                requested_token: 0,
                requested_amount: 50,
                nonce: 2, // should be 0 (nonce doesn't increment in tests)
            };
            // bob signs this using bob_keyring to create a signed_offer
            let signed_offer = SignedOffer {
                offer: offer.clone(),
                signer: bob.clone(),
                signature: Signature::from(bob_keyring.sign(&offer.encode())),
            };
            // make sure swap is ok
            assert_noop!(
                PRC20::swap(Origin::signed(alice.clone()), signed_offer),
                Error::<Test>::IncorrectNonce
            );
            // Bob has 10000 token 1
            assert_eq!(PRC20::balance_of((1, bob.clone())), 10000);
            // Alice has 100 token 1
            assert_eq!(PRC20::balance_of((0, alice)), 10000);
        });
    }

    #[test]
    fn multi_transfer_works() {
        ExtBuilder::build().execute_with(|| {
            // get account id for alice and bob
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            let charlie = AccountId::from(AccountKeyring::Charlie);
            // Alice creates token 0
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // create first transfer details struct
            let first_transfer = TokenTransferDetails {
                amount: 5,
                to: bob.clone(),
            };
            // create second transfer details struct
            let second_transfer = TokenTransferDetails {
                amount: 5,
                to: charlie.clone(),
            };
            // Create a vector of these transfer details struct
            let transfer_vec = vec![first_transfer, second_transfer];
            // do a multi transfer from alice's account
            assert_ok!(PRC20::multi_transfer(
                Origin::signed(alice.clone()),
                0,
                transfer_vec
            ));
            // Bob has 5 token 0
            assert_eq!(PRC20::balance_of((0, bob)), 5);
            // Charlie has 5 token 0
            assert_eq!(PRC20::balance_of((0, charlie)), 5);
            // Alice has 9990 token 0
            assert_eq!(PRC20::balance_of((0, alice)), 9990);
        });
    }

    #[test]
    fn multi_transfer_fails_sequentially() {
        // this basically means if the user does not have enough
        // balances for all transfers, the first transfers in the vec
        // have priority and go through
        ExtBuilder::build().execute_with(|| {
            // get account id for alice and bob
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            let charlie = AccountId::from(AccountKeyring::Charlie);
            // Alice creates token 0
            assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
            // create first transfer details struct
            let first_transfer = TokenTransferDetails {
                amount: 10000,
                to: bob.clone(),
            };
            // create second transfer details struct
            let second_transfer = TokenTransferDetails {
                amount: 5,
                to: charlie.clone(),
            };
            // Create a vector of these transfer details struct
            let transfer_vec = vec![first_transfer, second_transfer];

            let _partial_transfer_result =
                PRC20::multi_transfer(Origin::signed(alice.clone()), 0, transfer_vec);
            // Bob has 10000 token 0
            assert_eq!(PRC20::balance_of((0, bob)), 10000);
            // Alice has 0 token 0
            assert_eq!(PRC20::balance_of((0, alice)), 0);
            // Charlie has 0 token 0 since the second transfer
            // in the vector should fail
            assert_eq!(PRC20::balance_of((0, charlie)), 0);
        });
    }
}
