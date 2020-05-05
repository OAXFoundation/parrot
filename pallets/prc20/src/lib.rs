#![cfg_attr(not(feature = "std"), no_std)]
//! #PRC20 Module
//! A runtime module for an ERC20 equivalent token standard + a few extra handy features!
//! Additonal Features:
//! 1) Atomic Swap : Allows swapping tokens with another user in a single tx
//! 2) Multi-transfer: Allows transfering tokens to multiple users, in one single tx
use codec::{Codec, Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    weights::SimpleDispatchInfo, Parameter,
};
use frame_system::{self as system, ensure_signed};
use sp_arithmetic::traits::BaseArithmetic;
use sp_runtime::traits::{
    CheckedAdd, CheckedSub, IdentifyAccount, Member, One, StaticLookup, Verify, Zero,
};
use sp_std::{convert::TryInto, if_std, vec::Vec};

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

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as PRC20 {
        /// storage maps and value needed for operating a token
        TotalSupply get(total_supply): map hasher(blake2_128_concat) T::TokenId => T::TokenBalance;
        Balances get(balance_of): map hasher(blake2_128_concat) (T::TokenId, T::AccountId) => T::TokenBalance;
        Allowance get(allowance_of): map hasher(blake2_128_concat) (T::TokenId, T::AccountId, T::AccountId) => T::TokenBalance;
        TokenCount get(token_count): T::TokenId;
    }
}

// The module's dispatch functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        ///create a new token, passing totalSupply, (currently creator will receive total supply)
        #[weight = SimpleDispatchInfo::FixedNormal(100_000_000)]
        fn create_token(origin, #[compact] total_supply: T::TokenBalance) -> DispatchResult{
          // ensure signed from the sender
          let sender = ensure_signed(origin)?;
          // count the current token id
          let current_id = Self::token_count();
          // add one to the id
          let next_id = current_id.checked_add(&One::one()).ok_or("overflow when adding new token")?;
          // in this example we send the total supply to the creator
          <Balances<T>>::insert((current_id, sender.clone()), total_supply);
          // Add the currency id and total supply
          <TotalSupply<T>>::insert(current_id, total_supply);
          // Update the token count
          <TokenCount<T>>::put(next_id);
          // Broadcast a NewToken event
          Self::deposit_event(RawEvent::NewToken(current_id, sender, total_supply));
          Ok(())

        }
        /// do transfers like erc20 ( TokenId, To, Amount)
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        fn transfer(origin,
          to: <T::Lookup as StaticLookup>::Source,
          #[compact] id: T::TokenId,
          #[compact] amount: T::TokenBalance
        )-> DispatchResult
        {
          let sender = ensure_signed(origin)?;
          // get the to address
          let to = T::Lookup::lookup(to)?;
          // ensure sending amount is not zero or warn
          ensure!(!amount.is_zero(), "transfer amount should be non-zero");
          // make the transfer
          Self::make_transfer(id, sender, to, amount)?;
          Ok(())
        }

        /// do approval like erc20 (TokenId, To, Amount)
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        fn approve(origin,
          spender: <T::Lookup as StaticLookup>::Source,
          #[compact] id: T::TokenId,
          #[compact] value: T::TokenBalance
        ) -> DispatchResult {
          let sender = ensure_signed(origin)?;
          let spender = T::Lookup::lookup(spender)?;
          // add to allowance (here we don't mind someone setting 0 allowance, so no need to check)
          <Allowance<T>>::insert((id, sender.clone(), spender.clone()), value);
          // broadcast a Approval Event
          Self::deposit_event(RawEvent::Approval(id, sender, spender, value));
          Ok(())
        }
        /// do transfer from (allowing approver to spend token)(TokenId, From, To, Amount)
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        fn transfer_from(origin,
          from: T::AccountId,
          to: T::AccountId,
            #[compact] id: T::TokenId,
            #[compact] value: T::TokenBalance
        )-> DispatchResult {
            let sender = ensure_signed(origin)?;
            // check allowance
            let allowance = Self::allowance_of((id, from.clone(), sender.clone()));
            // check new allowance if transfer is made
            let updated_allowance = allowance.checked_sub(&value).ok_or("underflow in calculating allowance")?;
            // make the transfer
            Self::make_transfer(id, from.clone(), to.clone(), value)?;
            // update the allowance
            <Allowance<T>>::insert((id, from, sender), updated_allowance);
            Ok(())
        }
        /// atomic swap functionality for tokens
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        fn swap(origin, signed_offer:  SignedOffer<T::Signature,T::AccountId, T::TokenBalance, T::TokenId>
        )-> DispatchResult	{
           let sender = ensure_signed(origin)?;
           // Ensure that the SignedOffer is signed correctly
          if Self::verify_offer_signature(signed_offer.clone()).is_ok(){
            if_std! {println!("Signature is a match! Lets trade!");};
            // Ensure that offer amount is non zero
            ensure!(!signed_offer.offer.offer_amount.is_zero(), "Offer amount should be non-zero");
            // Ensure that requested amount is non zero
            ensure!(!signed_offer.offer.requested_amount.is_zero(), "Requested amount should be non-zero");
            // Make the Swap
            Self::make_swap(sender, signed_offer)?;
          };
          Ok(())
           }
         /// multi transfer functionality for tokens
         #[weight = SimpleDispatchInfo::FixedOperational(10_000_000)]
         fn multi_transfer(origin, #[compact] id: T::TokenId, td_vec: Vec<TokenTransferDetails<T::AccountId, T::TokenBalance>>) -> DispatchResult{
         // eval: Vec<T::TokenTransferDetails<T::AccountId, T::TokenBalance>>
            let sender = ensure_signed(origin)?;
            if_std!{println!("Multi transfer {:#?}", sender);};
            let num_transfers = td_vec.len();
            for i in 0..num_transfers{
                if_std!{println!("{:#?}", &td_vec[i])};

              // ensure sending amount is not zero or warn
              ensure!(!&td_vec[i].amount.is_zero(), "transfer amount should be non-zero");
              // make the transfer
              Self::make_transfer(id, sender.clone(), td_vec[i].to.clone(), td_vec[i].amount.clone())?;
            }
            // TODO, broadcast an event with status of each transfer!
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
        // event for a new token creation
        NewToken(TokenId, AccountId, TokenBalance),
        // event for a simple token transfer
        Transfer(TokenId, AccountId, AccountId, TokenBalance),
        // event for approval
        Approval(TokenId, AccountId, AccountId, TokenBalance),
        // event for Swap
        Swap(
            TokenId,
            TokenBalance,
            TokenId,
            TokenBalance,
            AccountId,
            AccountId,
        ),
        // Event for MultiTransfer
        MultiTransfer(Vec<(AccountId, TokenBalance, bool)>),
    }
);

impl<T: Trait> Module<T> {
    ///transfer
    fn make_transfer(
        id: T::TokenId,
        from: T::AccountId,
        to: T::AccountId,
        amount: T::TokenBalance,
    ) -> DispatchResult {
        // get balance of account
        let from_balance = Self::balance_of((id, from.clone()));
        // ensure user has enough tokens
        ensure!(from_balance >= amount, "user does not have enough tokens");
        // modify sender and receiver balance map
        <Balances<T>>::insert((id, from.clone()), from_balance - amount);
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
        // ensure has enough tokens
        ensure!(
            offer_from_balance >= signed_offer.offer.offer_amount,
            "Offerer does not have enough tokens"
        );
        // Check from balance of requestor
        let requested_from_balance =
            Self::balance_of((signed_offer.offer.requested_token, sender.clone()));
        // ensure has enough tokens
        ensure!(
            requested_from_balance >= signed_offer.offer.requested_amount,
            "Requestor does not have enough tokens"
        );
        // get maker nonce
        let maker_nonce: u128 =
            TryInto::<u128>::try_into(<system::Module<T>>::account_nonce(&signed_offer.signer))
                .map_err(|_| "error")?;
        // let taker_nonce: u128 =  TryInto::<u128>::try_into(<system::Module<T>>::account_nonce(&sender)).map_err(|_| "error")?;
        if_std! {println!("Maker Nonce: {:?}", maker_nonce.clone());
        //  println!("Taker Nonce: {:?}", taker_nonce);
         println!("Signed Offer Nonce: {:?}", signed_offer.offer.nonce.clone());
        }
        // ensure maker nonce is correct (replay protection)
        ensure!(
            maker_nonce == signed_offer.offer.nonce,
            "Nonce is incorrect!"
        );
        // modify sender and receiver balance map
        <Balances<T>>::insert(
            (signed_offer.offer.offer_token, signed_offer.signer.clone()),
            offer_from_balance - signed_offer.offer.offer_amount,
        );
        <Balances<T>>::mutate(
            (signed_offer.offer.offer_token, sender.clone()),
            |balance| *balance += signed_offer.offer.offer_amount,
        );

        <Balances<T>>::insert(
            (signed_offer.offer.requested_token, sender.clone()),
            requested_from_balance - signed_offer.offer.requested_amount,
        );
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
        assert_noop, assert_ok, impl_outer_origin, parameter_types, weights::Weight,
    };
    // use node_primitives::{AccountId, Signature};
    use sp_core::sr25519;
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };

    use substrate_test_client::{self, AccountKeyring};
    impl_outer_origin! {
      pub enum Origin for TestRuntime {}
    }

    /// The signature type used by accounts/transactions.
    pub type Signature = sr25519::Signature;
    /// An identifier for an account on this system.
    pub type AccountId = <Signature as Verify>::Signer;

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct TestRuntime;

    impl Trait for TestRuntime {
        type Event = ();
        type TokenBalance = u128;
        type TokenId = u128;
        type Public = AccountId;
        type Signature = Signature;
    }
    pub type PRC20 = Module<TestRuntime>;

    parameter_types! {
      pub const BlockHashCount: u64 = 250;
      pub const MaximumBlockWeight: Weight = 1024;
      pub const MaximumBlockLength: u32 = 2 * 1024;
      pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }

    impl system::Trait for TestRuntime {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
    }

    pub struct ExtBuilder;

    parameter_types! {
      pub const ExistentialDeposit: u64 = 1;
      pub const TransferFee: u64 = 0;
      pub const CreationFee: u64 = 0;
    }

    impl pallet_balances::Trait for TestRuntime {
        type Balance = u64;
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type DustRemoval = ();
    }

    type System = frame_system::Module<TestRuntime>;
    impl ExtBuilder {
        pub fn build() -> sp_io::TestExternalities {
            let mut t = system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();
            pallet_balances::GenesisConfig::<TestRuntime> { balances: vec![] }
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
            // Make sure the creator has 10000 tokens
            assert_eq!(PRC20::balance_of((0, alice.clone())), 10000);
            // Make sure reciever has 0 tokens
            assert_eq!(PRC20::balance_of((0, bob.clone())), 0);
            // make sure transfer goes ok and transfer
            assert_ok!(PRC20::transfer(Origin::signed(alice), bob.clone(), 0, 10));
            // make sure reciever balance is 10
            assert_eq!(PRC20::balance_of((0, bob)), 10);
        });
    }
    // TODO: fix this
    // #[test]
    // fn transfer_fails_with_no_balance(){
    //   let alice = AccountId::from(AccountKeyring::Alice);
    //   let bob = AccountId::from(AccountKeyring::Bob);
    // 	// Create Token
    // 	assert_ok!(PRC20::create_token(Origin::signed(alice.clone()), 10000));
    //   // Fails cuz bob does not have any tokens
    // 	assert_noop!(PRC20::transfer(Origin::signed(bob), alice, 0, 10), "user does not have enough tokens");
    // }

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
            // assert_eq!( TestRuntime::check_nonce(&bob), 2);
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

            // Now bob creates an offer struct (invalid since Bob owns token 1 and not 0)
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
                "Offerer does not have enough tokens"
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
                "Nonce is incorrect!"
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
        // this basically means if the user does not have enough balances for all transfers, the first transfers in the vec have priority and go through
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
            // Charlie has 0 token 0 since the second transfer in the vector should fail
            assert_eq!(PRC20::balance_of((0, charlie)), 0);
        });
    }
}
