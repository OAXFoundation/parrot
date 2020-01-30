/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult};
use system::ensure_signed;
use codec::{Decode, Encode};
use sp_std::vec::Vec;
use crate::Balance;



/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct TransferDetails< AccountId, Balance> {
    pub amount: Balance,
    pub to: AccountId,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
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
		pub fn multi_transfer(origin, _td_vec: Vec<TransferDetails<T::AccountId, Balance>>) -> DispatchResult {
			// check if signed
			let _who = ensure_signed(origin)?;
			// TODO: do multi transfer

    		// TODO: here we have to raise the right event
//			Self::deposit_event(RawEvent::SomethingStored(something, who));
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
    }
    type MiscModule = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
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



            // 1 has 50000000000

            // 2 didnt recieve anything since 0 ran out

            // 0 has total - 50000000000

        });
    }
}
