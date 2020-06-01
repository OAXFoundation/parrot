#![cfg_attr(not(feature = "std"), no_std)]
//! # Burner Module
//! Simple module that has a pot of funds with no private key, these funds are burnt every BurnPeriod,
//! and there is no way for someone to spend these funds
use frame_support::{
    decl_event, decl_module, decl_storage,
    traits::{Currency, Get, Imbalance, OnUnbalanced, ReservableCurrency},
};
use frame_system::{self as system};
use sp_runtime::{
    traits::{AccountIdConversion, Saturating, Zero},
    ModuleId,
};
use sp_std::if_std;

/// The Burner's module id, used for deriving its sovereign account ID.
const MODULE_ID: ModuleId = ModuleId(*b"py/burns");

/// Types necessary to enable using currency
type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + pallet_balances::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Currency type to use blockchains native currency
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    /// Period between successive burns. This is set in bin/runtime/lib.rs
    type BurnPeriod: Get<Self::BlockNumber>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Burner {
        Key get(fn key) config(): T::AccountId;
    }
    // Extra genesis so the account is started with min balance during geneses (avoid dust collection)
    add_extra_genesis {
        build(|_config| {
            // Create Burn account
            let _ = T::Currency::make_free_balance_be(
                &<Module<T>>::account_id(),
                T::Currency::minimum_balance(),
            );
        });
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
    {
        /// Balance burn event
        Burn(Balance),
        /// Some funds have been deposited.
        Deposit(Balance),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;
        /// Run on every block finalize
        fn on_finalize(n: T::BlockNumber) {
            // Run if BurnPeriod
            if (n % T::BurnPeriod::get()).is_zero() {
            Self::burn();
            }
        }
    }
}

impl<T: Trait> Module<T> {
    // Add public immutables and private mutables.
    /// The account ID of the Burner pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn account_id() -> T::AccountId {
        MODULE_ID.into_account()
    }
    /// Return the amount of money in the burn pot (substrats existential deposit so will only show available balance, not free balance
    /// The existential deposit is not part of the pot so burn account never gets deleted.
    fn pot() -> BalanceOf<T> {
        T::Currency::free_balance(&Self::account_id())
            // Must never be less than 0 but better be safe.
            .saturating_sub(T::Currency::minimum_balance())
    }
    /// Function that burns funds in the pot
    fn burn() {
        let budget_remaining = Self::pot();
        if_std! {println!{"Budget remaining: {:#?} !", budget_remaining}};
        let _ = T::Currency::slash(&Self::account_id(), budget_remaining);
        Self::deposit_event(RawEvent::Burn(budget_remaining));
    }
}

// This is to accept incoming deposits for fee payment, and broadcast a Deposit event
impl<T: Trait> OnUnbalanced<NegativeImbalanceOf<T>> for Module<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
        let numeric_amount = amount.peek();
        // Must resolve into existing but better to be safe.
        let _ = T::Currency::resolve_creating(&Self::account_id(), amount);
        Self::deposit_event(RawEvent::Deposit(numeric_amount));
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };

    mod burn {
        pub use super::super::*;
    }

    impl_outer_origin! {
        pub enum Origin for Test  where system = frame_system {}
    }

    impl_outer_event! {
        pub enum Event for Test {
            system<T>,
            pallet_balances<T>,
            burn<T>,
        }
    }

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
        type Index = u64;
        type BlockNumber = u64;
        type Call = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
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
            pub const ExistentialDeposit: u64 = 1;
    }
    // implement balances trait for Test
    impl pallet_balances::Trait for Test {
        type Balance = u64;
        type Event = Event;
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type DustRemoval = ();
    }

    // implement the burn trait for Test
    parameter_types! {
        pub const BurnPeriod: u64 = 2;
    }
    impl Trait for Test {
        type Currency = pallet_balances::Module<Test>;
        type Event = Event;
        type BurnPeriod = BurnPeriod;
    }

    type System = frame_system::Module<Test>;
    type Balances = pallet_balances::Module<Test>;
    type Burner = Module<Test>;

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(0, 10000)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }

    #[test]
    fn genesis_config_works() {
        new_test_ext().execute_with(|| {
            assert_eq!(Burner::pot(), 0);
        });
    }

    #[test]
    fn pot_returns_correct_usable_amount() {
        new_test_ext().execute_with(|| {
            // set burner balance to 101
            Balances::make_free_balance_be(&Burner::account_id(), 101);
            // make sure pot returns 100  since ExistentialDeposit =1 . 101-1 = 100
            assert_eq!(Burner::pot(), 100);
        });
    }

    #[test]
    fn burn_works() {
        new_test_ext().execute_with(|| {
            assert_eq!(Balances::total_issuance(), 10000);
            // set burner balance to 101
            Balances::make_free_balance_be(&Burner::account_id(), 101);
            assert_eq!(Balances::total_issuance(), 10101);
            // run burn
            Burner::burn();
            // make sure pot returns 0 now
            assert_eq!(Burner::pot(), 0);
            assert_eq!(Balances::total_issuance(), 10001);
        });
    }

    #[test]
    fn burn_works2() {
        new_test_ext().execute_with(|| {
            let burn_amount = 200;
            Balances::make_free_balance_be(&Burner::account_id(), burn_amount);
            let total_supply = Balances::total_issuance();
            let avail_burn = Burner::pot();
            // perform the burn
            Burner::burn();
            // assert that there is nothing left to burn
            assert_eq!(Burner::pot(), 0);
            // assert that the total_issuance decreased by the amount we burned
            assert_eq!(total_supply - avail_burn, Balances::total_issuance());
        })
    }

    #[test]
    fn existential_deposit_amount() {
        new_test_ext().execute_with(|| {
            let burn_amount = 200;
            Balances::make_free_balance_be(&Burner::account_id(), burn_amount);
            // burn cannot leave the balance totally empty due to existential deposit requirement
            // so we expect there to be 1 token left after burning so test that it doesn't burn everything
            assert_eq!(Burner::pot(), burn_amount - 1);
        })
    }
}
