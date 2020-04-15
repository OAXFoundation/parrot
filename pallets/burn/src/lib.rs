// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! # Burner Module
//! Simple module that has a pot of funds with no private key, these funds are burnt every BurnPeriod
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
use sp_std::prelude::*;
use frame_support::{decl_module, decl_storage, decl_event};
use frame_support::traits::{
    Currency, Imbalance, OnUnbalanced,
    ReservableCurrency, Get
};
use sp_runtime::{ ModuleId, traits::{
    AccountIdConversion, Saturating, Zero
}};
use sp_std::if_std;
use frame_system::{self as system};

/// The Burner's module id, used for deriving its sovereign account ID.
const MODULE_ID: ModuleId = ModuleId(*b"py/burns");

// We need the balance module enabled (add this based on balance/src/lib.rs
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + pallet_balances::Trait{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    // Period between successive burns. This is set in bin/runtime/lib.rs
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
		Deposit(Balance ),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;
        // Run on every block finalize
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
    /// Return the amount of money in the burn pot (substrats exestiential deposit so will only show available balance, not free balance
    // The existential deposit is not part of the pot so burn account never gets deleted.
    fn pot() -> BalanceOf<T> {
        T::Currency::free_balance(&Self::account_id())
            // Must never be less than 0 but better be safe.
            .saturating_sub(T::Currency::minimum_balance())
    }
    // Burn the funds
    fn burn() {
        let budget_remaining = Self::pot();
        if_std! {println!{"Budget remaining: {:#?} !", budget_remaining}};
        if_std! {println!{"BURN BABY BURN!"}};
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






#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{assert_noop, assert_ok, impl_outer_origin, parameter_types, weights::Weight};
    use frame_support::traits::Contains;
    use sp_core::H256;
    use sp_runtime::{
        Perbill,
        testing::Header,
        traits::{BlakeTwo256, OnFinalize, IdentityLookup, BadOrigin},
    };


    impl_outer_origin! {
		pub enum Origin for Test  where system = frame_system {}
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
        type Index = u64;
        type BlockNumber = u64;
        type Call = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
        type Version = ();
        type ModuleToIndex = ();
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
    }
    parameter_types! {
		pub const ExistentialDeposit: u64 = 1;
}
    impl pallet_balances::Trait for Test {
        type Balance = u64;
        type Event = ();
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
    }
    parameter_types! {
		pub const BurnPeriod: u64 = 2;
	}


    impl Trait for Test {
        type Currency = pallet_balances::Module<Test>;
        type Event = ();
        type BurnPeriod = BurnPeriod;
    }

    type System = frame_system::Module<Test>;
    type Balances = pallet_balances::Module<Test>;
    type Burner = Module<Test>;


    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(0,10000)],
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










