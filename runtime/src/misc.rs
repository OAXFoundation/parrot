/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult, ensure, traits::{Currency, ExistenceRequirement}};
use system::ensure_signed;
use codec::{Decode, Encode, HasCompact};
use sp_std::vec::Vec;
use crate::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::Zero;
use sp_std::if_std;
// TODO: figure out what this import should actually be and how to bring Balance into scope, since currently the transfer function has the following error :  expected associated type, found `u128`
use crate::Balance;

//TODO: this is from society frame, but cant get this to compile
//use frame_system::{self as system, ensure_signed, ensure_root};
//type BalanceOf<T, I> = <<T as Trait<I>>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}


#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct TransferDetails< AccountId, Balance: HasCompact> {
    pub amount: Balance,
    pub to: AccountId,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as MiscModule {
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;


        // Multi transfer event
		pub fn multi_transfer(origin, td_vec: Vec<TransferDetails<T::AccountId, Balance>>) -> DispatchResult {
			// check if signed
			let sender = ensure_signed(origin)?;
			// iterate and do transfers

            let num_transfers = td_vec.len();
            for i in 0..num_transfers{
                  //if_std!{println!("{:#?}", &td_vec[i])};
                  // ensure sending amount is not zero or warn
                  ensure!(!&td_vec[i].amount.is_zero(), "transfer amount should be non-zero");
                  // TODO: make the transfer (currently has issues with the balance type)
                  let balance: Balance = td_vec[i].amount.clone();
                  // TODO: uncommenting this line causes a type mismatch error (this may be due to the import of Balance crate, please look at Society crate to evaluate how we should bring "Balance" into scope
                  //T::Currency::transfer( &sender.clone(), &td_vec[i].to.clone(), balance, ExistenceRequirement::AllowDeath);

        }
    		// Since this is going to trigger transfers that already trigger events, we do not need to trigger a multitransfer event.
    		// Another reason for not triggering an event is that these can fail sequentially, meaning the first 2 transfers go through and the 3rd fail due to insufficient balances.
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use sp_core::H256;
    use frame_support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
    };

    impl_outer_origin! {
		pub enum Origin for Test {}
	}

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
		pub const ExistentialDeposit: u64 = 0;
	pub const TransferFee: u64 = 0;
	pub const CreationFee: u64 = 0;
	}
    impl system::Trait for Test {
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
    }



    impl Trait for Test {
        type Event = ();
        type Currency = balances::Module<Self>;
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type TransferPayment = ();
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
    }



    type MiscModule = Module<Test>;



    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        // TODO add genesis config and give account 0 all the native currency
        let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(0,10000)],
            vesting: vec![],
        }
            .assimilate_storage(&mut t)
            .unwrap();
        t.into()
    }



    #[test]
    fn multi_transfer_works() {
        new_test_ext().execute_with(|| {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42

            // TODO: start with account 0 with all native tokens in genesis config

            // create first transfer details struct
            let first_transfer = TransferDetails{
                amount: 10000,
                to: 1,
            };
            // create second transfer details struct
            let second_transfer = TransferDetails{
                amount: 5,
                to:2,
            };
            // Create a vector of these transfer details struct
            let transfer_vec = vec![first_transfer, second_transfer];
            // Do a multitransfer
            assert_ok!(MiscModule::multi_transfer(Origin::signed(0), transfer_vec));

            // TODO: assert__eq balances

            // 1 has 10000 balance

            // 2 has 5 balance

            // 0 has total - 10005 balance

        });
    }

    #[test]
    fn multi_transfer_fails_sequentiallyy() {
        new_test_ext().execute_with(|| {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42

            // TODO: start with account 0 with all native tokens in genesis config

            // create first transfer details struct
            let first_transfer = TransferDetails{
                amount: 50000000000,
                to: 1,
            };
            // create second transfer details struct
            let second_transfer = TransferDetails{
                amount: 50000000000,
                to:2,
            };
            // Create a vector of these transfer details struct
            let transfer_vec = vec![first_transfer, second_transfer];

            assert_ok!(MiscModule::multi_transfer(Origin::signed(0), transfer_vec));

            // TODO: assert_eq balances
            // 1 has 50000000000

            // 2 didnt recieve anything since 0 ran out

            // 0 has total - 50000000000

        });
    }
}
