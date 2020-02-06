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

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: system::Trait + pallet_balances::Trait{
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
		pub fn multi_transfer(origin, td_vec: Vec<TransferDetails<T::AccountId, BalanceOf<T>>>) -> DispatchResult {
			// check if signed
			let sender = ensure_signed(origin)?;
			// iterate and do transfers

            let num_transfers = td_vec.len();
            for i in 0..num_transfers{
                  //if_std!{println!("{:#?}", &td_vec[i])};
                  //TODO: do we want to do more ensure? or do we want to save costs, I think we should focus on saving costs here

                  // ensure sending amount is not zero or warn
                  ensure!(!&td_vec[i].amount.is_zero(), "transfer amount should be non-zero");
                  // make the transfer
                  T::Currency::transfer( &sender.clone(), &td_vec[i].to.clone(), td_vec[i].amount.clone(), ExistenceRequirement::AllowDeath);

        }
    		// Since this is going to trigger transfers that already trigger events, we do not need to trigger a multitransfer event.
    		// Another reason for not triggering an event is that these can fail sequentially, meaning the first 2 transfers go through and the 3rd fail due to insufficient balances.
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where
	AccountId = <T as system::Trait>::AccountId,
    Balance = BalanceOf<T>
    {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		// TODO: We are choosing not to broadcast anything here since things may feel sequentially. Look into if this can be removed, since this macro may expand to a few 100 lines of code
		SomethingStored(u32, AccountId, Balance),
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
        type Currency = pallet_balances::Module<Self>;
    }
    impl pallet_balances::Trait for Test {
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
    pub type Balances = pallet_balances::Module<Test>;



    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
        pallet_balances::GenesisConfig::<Test> {
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

            // create first transfer details struct
            let first_transfer = TransferDetails{
                amount: 1000,
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
    fn multi_transfer_fails_sequentiallyy() {
        new_test_ext().execute_with(|| {

            // create first transfer details struct
            let first_transfer = TransferDetails{
                amount: 10000,
                to: 1,
            };
            // create second transfer details struct
            let second_transfer = TransferDetails{
                amount: 100,
                to:2,
            };
            // Create a vector of these transfer details struct
            let transfer_vec = vec![first_transfer, second_transfer];

            assert_ok!(MiscModule::multi_transfer(Origin::signed(0), transfer_vec));

            //assert_eq balances

            // 1 has 50000000000
            assert_eq!(Balances::free_balance(1), 10000);
            // 2 didnt recieve anything since 0 ran out
            assert_eq!(Balances::free_balance(2), 0);
            // 0 has total - 0
            assert_eq!(Balances::free_balance(0), 0);
        });
    }
}
