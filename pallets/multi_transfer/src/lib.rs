#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};
use frame_support::weights::SimpleDispatchInfo;
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement},
};
/// A runtime module for doing multi-transfers!
use sp_runtime::traits::Zero;
// the decl_event macros expands to look for system, and its actually called frame_system here so this line makes life easier
use frame_system as system;
use sp_std::if_std;
use sp_std::vec::Vec;
use system::ensure_signed;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + pallet_balances::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}

// This is used to encode each transfer in a multiTransfer
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct TransferDetails<AccountId, Balance: HasCompact> {
    pub amount: Balance,
    pub to: AccountId,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiscModule {
    }
}

// The module's dispatch functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        // Multi transfer event
        #[weight = SimpleDispatchInfo::FixedOperational(10_000_000)]
        pub fn multi_transfer(origin, td_vec: Vec<TransferDetails<T::AccountId, BalanceOf<T>>>) -> DispatchResult {
            // check if signed
            let sender = ensure_signed(origin)?;
            // get total number
            let num_transfers = td_vec.len();
            //TODO: limit this to a certain amount of multiTransfers

            // build a status vector, to push status of each transfer
            let mut status_vector: Vec<(T::AccountId, BalanceOf<T>, bool)> = Vec::new();

            for i in 0..num_transfers{
                  //if_std!{println!("{:#?}", &td_vec[i])};
                  //TODO: do we want to do more ensured?

                  // ensure sending amount is not zero or warn
                  ensure!(!&td_vec[i].amount.is_zero(), "transfer amount should be non-zero");
                  // make the transfer and get the result
                  let transfer_result = T::Currency::transfer( &sender.clone(), &td_vec[i].to.clone(), td_vec[i].amount.clone(), ExistenceRequirement::AllowDeath);
                  // log the transfer result
                  if_std!{println!("{:#?}", transfer_result)}
                  // get the status as either true or false
                  let transfer_status = match transfer_result {
                   Ok(()) => true,
                   Err(_e) => false
                  };

                  status_vector.push((td_vec[i].to.clone(), td_vec[i].amount, transfer_status));

        }
           // if_std!{println!("{:#?}", status_vector)}
           // trigger a multi-transfer event.
            Self::deposit_event(RawEvent::MultiTransfer(status_vector));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit function, from our runtime functions
        SomethingStored(u32, AccountId, Balance),
        MultiTransfer(Vec<(AccountId, Balance, bool)>),
    }
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::traits::{Get, IsDeadAccount};
    use frame_support::{assert_ok, impl_outer_origin, parameter_types, weights::Weight};
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };
    use std::cell::RefCell;

    impl_outer_origin! {
        pub enum Origin for Test {}
    }
    thread_local! {
        static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
    }

    pub struct ExistentialDeposit;
    impl Get<u64> for ExistentialDeposit {
        fn get() -> u64 {
            EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
        }
    }

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
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
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

    impl pallet_balances::Trait for Test {
        type Balance = u64;
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type DustRemoval = ();
    }
    impl Trait for Test {
        type Event = ();
        type Currency = pallet_balances::Module<Self>;
    }
    type System = frame_system::Module<Test>;

    type MiscModule = Module<Test>;
    pub type Balances = pallet_balances::Module<Test>;

    pub struct ExtBuilder {
        existential_deposit: u64,
        monied: bool,
    }
    impl Default for ExtBuilder {
        fn default() -> Self {
            Self {
                existential_deposit: 1,
                monied: false,
            }
        }
    }
    impl ExtBuilder {
        pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
            self.existential_deposit = existential_deposit;
            self
        }
        pub fn monied(mut self, monied: bool) -> Self {
            self.monied = monied;
            if self.existential_deposit == 0 {
                self.existential_deposit = 1;
            }
            self
        }
        pub fn set_associated_consts(&self) {
            EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        }
        pub fn build(self) -> sp_io::TestExternalities {
            self.set_associated_consts();
            let mut t = frame_system::GenesisConfig::default()
                .build_storage::<Test>()
                .unwrap();
            pallet_balances::GenesisConfig::<Test> {
                balances: if self.monied {
                    vec![(0, 10000)]
                } else {
                    vec![]
                },
            }
            .assimilate_storage(&mut t)
            .unwrap();
            t.into()
        }
    }

    #[test]
    fn multi_transfer_works() {
        ExtBuilder::default()
            .existential_deposit(1)
            .monied(true)
            .build()
            .execute_with(|| {
                // create first transfer details struct
                let first_transfer = TransferDetails {
                    amount: 1000,
                    to: 1,
                };
                // create second transfer details struct
                let second_transfer = TransferDetails { amount: 5, to: 2 };
                // Create a vector of these transfer details struct
                let transfer_vec = vec![first_transfer, second_transfer];
                // Do a multitransfer
                assert_ok!(MiscModule::multi_transfer(Origin::signed(0), transfer_vec));

                //assert_eq balances

                // 1 has 1000 balance
                assert_eq!(Balances::free_balance(1), 1000);
                // 2 has 5 balance
                assert_eq!(Balances::free_balance(2), 5);
                // 0 has total - 1005 balance
                assert_eq!(Balances::free_balance(0), 8995);
            });
    }

    #[test]
    fn multi_transfer_fails_sequentially() {
        ExtBuilder::default()
            .existential_deposit(1)
            .monied(true)
            .build()
            .execute_with(|| {
                // create first transfer details struct
                let first_transfer = TransferDetails {
                    amount: 10000,
                    to: 1,
                };
                // create second transfer details struct
                let second_transfer = TransferDetails { amount: 100, to: 2 };
                // Create a vector of these transfer details struct
                let transfer_vec = vec![first_transfer, second_transfer];
                // Do the multi transfer (#note the sender is being stupid here and his account may be destroyed due to existential deposit requirements)
                assert_ok!(MiscModule::multi_transfer(Origin::signed(0), transfer_vec));

                //0 should not exist anymore due to existential deposit (#TODO: maybe we want different behaviour)
                assert_eq!(Balances::is_dead_account(&0), true);

                //assert_eq balances

                // 1 should have 10000
                assert_eq!(Balances::free_balance(1), 10000);
                // 2 didnt receive anything since 0 ran out
                assert_eq!(Balances::free_balance(2), 0);
                // 0 has total - 0
                assert_eq!(Balances::free_balance(0), 0);
            });
    }
}
