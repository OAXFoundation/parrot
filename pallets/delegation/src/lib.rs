#![cfg_attr(not(feature = "std"), no_std)]
//! # Fee Delegation Module
//! Simple module that allows a user to pay for another users transfer
//! A user first signs a message offline and sends it to the fee delegator.
//! If the fee delegator wants to broadcast this message he
//! may choose to do so, he will be charged a fee for the users transfer
//!  instead of the user This basically achieves a free transfer for the user
use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement, Get},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{IdentifyAccount, Member, Verify};
use sp_std::{convert::TryInto, if_std};

/// Types necessary to enable using currency
type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + pallet_balances::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Currency type to use blockchain native currency
    type Currency: Currency<Self::AccountId>;
    /// Additional types for verifying offline signatures in delegated methods
    type Public: IdentifyAccount<AccountId = Self::AccountId>;
    type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
}

/// This is used to encode each transfer, for a delegated Transfer
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct DelegatedTransferDetails<AccountId, Balance> {
    pub amount: Balance,
    pub to: AccountId,
    pub nonce: u128,
}

/// This is the signed version of the delegated Transfer
/// that is sent to the fee delegator after signing
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct SignedDelegatedTransferDetails<Signature, AccountId, Balance> {
    pub transfer: DelegatedTransferDetails<AccountId, Balance>,
    pub signature: Signature,
    pub signer: AccountId,
}

// The modules error types
decl_error! {
    pub enum Error for Module<T: Trait>{
        /// Wrong Offline Signature
        InvalidSignature,
        /// Wrong Nonce
        IncorrectNonce,
    }
}

// The module's dispatch functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        /// this is needed only if you are using events in your module
        fn deposit_event() = default;

        /// Delegated transfer
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 70_000_000]
        fn delegated_transfer(origin,
            signed_dtd:  SignedDelegatedTransferDetails<T::Signature,
                T::AccountId,
                BalanceOf<T>>
        )-> DispatchResult	{
            // Ensure signed by fee delegator
            let delegator = ensure_signed(origin)?;
            // Ensure signed by user who wants to send funds, or return error
            ensure!(Self::verify_dtd_signature(signed_dtd.clone()).is_ok(),
                <Error<T>>::InvalidSignature);
            // get the sender's nonce
            let sender_nonce: u128 = TryInto::<u128>::try_into(
                <system::Module<T>>::account_nonce(&signed_dtd.signer))
                .map_err(|_| "error")?;
            // verify his nonce is correct,
            // or return error (
            // this allows replay protection of the offline signed messaged)
            ensure!(sender_nonce == signed_dtd.transfer.nonce,
                <Error<T>>::IncorrectNonce);
            // make the transfer
            let transfer_result = T::Currency::transfer(&signed_dtd.signer,
                &signed_dtd.transfer.to,
                signed_dtd.transfer.amount,
                ExistenceRequirement::KeepAlive);
            // get the status of the transfer, if success
            // increment sender nonce + broadcast event , return the error
            match transfer_result {
                Ok(()) => {
                    // log the transfer result
                    if_std! {println!("{:#?}", transfer_result)}
                    // increment account nonce for replay protection
                    <system::Module<T>>::inc_account_nonce(
                        &signed_dtd.signer);
                    // broadcast an event
                    Self::deposit_event(RawEvent::DelegatedTransfer(
                        delegator,
                        signed_dtd.signer,
                        signed_dtd.transfer.to,
                        signed_dtd.transfer.amount,
                    ));
                    Ok(())
                },
                // just propagate the balances error
                Err(e) => Err(e),
            }
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// DelegatedTransfer Event with details below
        /// DelegatedTransfer(DelegatorAddr, SenderAddr, ReceiverAddr, Amount)
        DelegatedTransfer(AccountId, AccountId, AccountId, Balance),
    }
);

/// custom functions for this module
impl<T: Trait> Module<T> {
    /// function to verify a signature given a signed Delegated Transfer
    fn verify_dtd_signature(
        signed_dtd: SignedDelegatedTransferDetails<T::Signature, T::AccountId, BalanceOf<T>>,
    ) -> Result<(), &'static str> {
        match signed_dtd
            .signature
            .verify(&signed_dtd.transfer.encode()[..], &signed_dtd.signer)
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

    mod delegation {
        pub use super::super::*;
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
    }
    impl_outer_event! {
        pub enum Event for Test {
            system<T>,
            pallet_balances<T>,
            delegation<T>,
        }
    }

    // The signature type used by accounts/transactions.
    pub type Signature = sr25519::Signature;
    // An identifier for an account on this system.
    pub type AccountId = <Signature as Verify>::Signer;

    // implement frame_system trait for test
    #[derive(Clone, Eq, PartialEq)]
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

    impl Trait for Test {
        type Event = Event;
        type Public = AccountId;
        type Signature = Signature;
        type Currency = pallet_balances::Module<Self>;
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
    type Balances = pallet_balances::Module<Test>;
    type DelegatorModule = Module<Test>;

    impl ExtBuilder {
        pub fn build() -> sp_io::TestExternalities {
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);

            let mut t = system::GenesisConfig::default()
                .build_storage::<Test>()
                .unwrap();
            pallet_balances::GenesisConfig::<Test> {
                balances: vec![(alice, 1000), (bob, 1000)],
            }
            .assimilate_storage(&mut t)
            .unwrap();
            t.into()
        }
    }

    #[test]
    fn delegated_transfer_works() {
        ExtBuilder::build().execute_with(|| {
            // Bob will be the user trying a free transfer to his friend Eve
            // Alice will act as the fee delegator interacting with the
            // blockchain
            // get account id for alice and bob and eve
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            let eve = AccountId::from(AccountKeyring::Eve);
            // get account keyring for Bob
            let bob_keyring = AccountKeyring::Bob;
            // Get everyone's initial balance
            let bob_balance = Balances::free_balance(&bob);
            let eve_balance = Balances::free_balance(&eve);
            let alice_balance = Balances::free_balance(&alice);
            // Set the transfer amount for this test
            let transfer_amount = 100;
            // Bob creates a delegated transfer detail
            let dtd = DelegatedTransferDetails {
                amount: transfer_amount,
                to: eve.clone(),
                nonce: 0,
            };
            // Bob signs this dtd
            let signed_dtd = SignedDelegatedTransferDetails {
                transfer: dtd.clone(),
                signer: bob.clone(),
                signature: Signature::from(bob_keyring.sign(&dtd.encode())),
            };
            // Fee delegator Alice broadcasts this transaction for Bob
            assert_ok!(DelegatorModule::delegated_transfer(
                Origin::signed(alice.clone()),
                signed_dtd
            ));
            //Bob balance is reduced
            assert_eq!(
                Balances::free_balance(bob.clone()),
                bob_balance - transfer_amount
            );
            // Eve's balance is increased
            assert_eq!(
                Balances::free_balance(eve.clone()),
                eve_balance + transfer_amount
            );
            // Alice's balance is unchanged since fee is not activated
            assert_eq!(Balances::free_balance(alice.clone()), alice_balance);
        });
    }

    #[test]
    fn delegated_transfer_fails_if_wrong_nonce() {
        ExtBuilder::build().execute_with(|| {
            // Alice will act as the fee delegator interacting with the
            // blockchain
            // get account id for alice and bob and eve
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            let eve = AccountId::from(AccountKeyring::Eve);
            // get account keyring for Eve
            let eve_keyring = AccountKeyring::Eve;
            // Get everyone's initial balance
            let bob_balance = Balances::free_balance(&bob);
            let eve_balance = Balances::free_balance(&eve);
            let alice_balance = Balances::free_balance(&alice);
            // Set the transfer amount for this test
            let transfer_amount = 100;
            // Eve creates a delegated transfer detail with the wrong nonce !
            let dtd = DelegatedTransferDetails {
                amount: transfer_amount,
                to: bob.clone(),
                nonce: 5,
            };
            // Eve signs this dtd
            let signed_dtd = SignedDelegatedTransferDetails {
                transfer: dtd.clone(),
                signer: eve.clone(),
                signature: Signature::from(eve_keyring.sign(&dtd.encode())),
            };
            // Fee delegator Alice broadcasts this transaction for Eve
            // Do a multi_transfer, assert it errors saying incorrect nonce
            assert_noop!(
                DelegatorModule::delegated_transfer(Origin::signed(alice.clone()), signed_dtd),
                Error::<Test>::IncorrectNonce
            );
            //Bob balance is unchanged
            assert_eq!(Balances::free_balance(bob.clone()), bob_balance);
            // Eve's balance is unchanged
            assert_eq!(Balances::free_balance(eve.clone()), eve_balance);
            // Alice's balance is unchanged since fee is not activated
            assert_eq!(Balances::free_balance(alice.clone()), alice_balance);
        });
    }
    #[test]
    fn delegated_transfer_fails_if_wrong_signature() {
        ExtBuilder::build().execute_with(|| {
            // Alice will act as the fee delegator interacting with the
            // blockchain
            // get account id for alice and bob and eve
            let alice = AccountId::from(AccountKeyring::Alice);
            let bob = AccountId::from(AccountKeyring::Bob);
            let eve = AccountId::from(AccountKeyring::Eve);
            // get account keyring for Bob
            let bob_keyring = AccountKeyring::Bob;
            // Get everyone's initial balance
            let bob_balance = Balances::free_balance(&bob);
            let eve_balance = Balances::free_balance(&eve);
            let alice_balance = Balances::free_balance(&alice);
            // Set the transfer amount for this test
            let transfer_amount = 100;
            // Eve creates a delegated transfer detail
            let dtd = DelegatedTransferDetails {
                amount: transfer_amount,
                to: bob.clone(),
                nonce: 5,
            };
            // Bob signs this dtd, for an invalid signature
            // (Eve should be the actual signer, but this
            // is a way to create a invalid sig)
            let signed_dtd = SignedDelegatedTransferDetails {
                transfer: dtd.clone(),
                signer: eve.clone(),
                signature: Signature::from(bob_keyring.sign(&dtd.encode())),
            };
            // Fee delegator Alice broadcasts this transaction for Eve
            // Do a multi_transfer, assert it errors saying wrong signature
            assert_noop!(
                DelegatorModule::delegated_transfer(Origin::signed(alice.clone()), signed_dtd),
                Error::<Test>::InvalidSignature
            );
            //Bob balance is unchanged
            assert_eq!(Balances::free_balance(bob.clone()), bob_balance);
            // Eve's balance is unchanged
            assert_eq!(Balances::free_balance(eve.clone()), eve_balance);
            // Alice's balance is unchanged since fee is not activated
            assert_eq!(Balances::free_balance(alice.clone()), alice_balance);
        });
    }
}
