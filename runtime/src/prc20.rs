/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, Parameter, ensure, dispatch::Result, traits::{Currency}};

use system::ensure_signed;
use runtime_primitives::traits::{Member, SimpleArithmetic, Zero, StaticLookup, One,
	CheckedAdd, CheckedSub, MaybeSerializeDebug,};

use support::codec::{Codec};
/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	// This is to track TokenBalances of each token on each account 
	type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + MaybeSerializeDebug;
	// This is to give each token a unique TokenId 
	type TokenId: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + MaybeSerializeDebug;
}

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as PRC20 {
		// ERC 20 Related Storage Items
		
		// total supply Map[TokenId] = TotalSupply 
		TotalSupply get(total_supply): map T::TokenId => T::TokenBalance;
		// balance tracker Map[TokenId, AccountId] = Balance
		Balances get(balance_of): map (T::TokenId, T::AccountId) => T::TokenBalance;
		// allowance, to track how much is approved, Map(TokenId, Sender, Spender) = ApprovedAmount
		Allowance get(allowance_of): map (T::TokenId, T::AccountId, T::AccountId) => T::TokenBalance;
		// Additional (TokenCount), to make sure we don't overflow while creating new tokens 
		TokenCount get(token_count): T::TokenId;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		//create a new token, passing totalSupply, (currently creator will receive total supply) 
		fn create_token(origin, #[compact] total_supply: T::TokenBalance) {
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
			Self::deposit_event(RawEvent::NewToken(current_id, sender.clone(), total_supply));
			}
		// do transfers like erc20 ( TokenId, To, Amount)
		fn transfer(origin,
			#[compact] id: T::TokenId,
			to: <T::Lookup as StaticLookup>::Source,
			#[compact] amount: T::TokenBalance
		) 
		{
			let sender = ensure_signed(origin)?;
			// get the to address 
			let to = T::Lookup::lookup(to)?;
			// ensure sending amount is not zero or warn
			ensure!(!amount.is_zero(), "transfer amount should be non-zero");
			// make the transfer 
			Self::make_transfer(id, sender, to, amount)?;
		}
		// do approval like erc20 (TokenId, To, Amount)
		fn approve(origin,
			#[compact] id: T::TokenId,
			spender: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::TokenBalance
		) {
			let sender = ensure_signed(origin)?;
			let spender = T::Lookup::lookup(spender)?;
			// add to allowance (here we don't mind someone setting 0 allowance, so no need to check)
			<Allowance<T>>::insert((id, sender.clone(), spender.clone()), value);
			// broadcast a Approval Event 
			Self::deposit_event(RawEvent::Approval(id, sender, spender, value));
		}
		// do transfer from (allowing approver to spend token)(TokenId, From, To, Amount)
        fn transfer_from(origin,
            #[compact] id: T::TokenId,
            from: T::AccountId,
            to: T::AccountId,
            #[compact] value: T::TokenBalance
        ) {
            let sender = ensure_signed(origin)?;
            // check allowance
            let allowance = Self::allowance_of((id, from.clone(), sender.clone()));
            // check new allowance if transfer is made 
            let updated_allowance = allowance.checked_sub(&value).ok_or("underflow in calculating allowance")?;
            // make the transfer 
            Self::make_transfer(id, from.clone(), to.clone(), value)?;
            // update the allowance 
            <Allowance<T>>::insert((id, from, sender), updated_allowance);
        }
	
	}
}

decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		TokenId = <T as Trait>::TokenId,
		TokenBalance = <T as Trait>::TokenBalance
		{
			// event for a new token creation 
			NewToken(TokenId, AccountId, TokenBalance),
			// event for a simple token transfer 
			Transfer(TokenId, AccountId, AccountId, TokenBalance),
			// event for approval
			Approval(TokenId, AccountId, AccountId, TokenBalance),
		}
	
);

impl<T: Trait> Module<T> {

	fn make_transfer(id: T::TokenId, from: T::AccountId, to: T::AccountId, amount: T::TokenBalance) -> Result {
		// get balance of account 
        let from_balance = Self::balance_of((id, from.clone()));
        // ensure user has enough tokens 
        ensure!(from_balance >= amount.clone(), "user does not have enough tokens");
        // modify sender and receiver balance map 
        <Balances<T>>::insert((id, from.clone()), from_balance - amount.clone());
        <Balances<T>>::mutate((id, to.clone()), |balance| *balance += amount.clone());
        // broadcast a transfer event 
        Self::deposit_event(RawEvent::Transfer(id, from, to, amount));


		Ok(())
	}
}

// /// tests for this module
// #[cfg(test)]
// mod tests {
// 	use super::*;

// 	use runtime_io::with_externalities;
// 	use primitives::{H256, Blake2Hasher};
// 	use support::{impl_outer_origin, assert_ok};
// 	use runtime_primitives::{
// 		BuildStorage,
// 		traits::{BlakeTwo256, IdentityLookup},
// 		testing::{Digest, DigestItem, Header}
// 	};

// 	impl_outer_origin! {
// 		pub enum Origin for Test {}
// 	}

// 	// For testing the module, we construct most of a mock runtime. This means
// 	// first constructing a configuration type (`Test`) which `impl`s each of the
// 	// configuration traits of modules we want to use.
// 	#[derive(Clone, Eq, PartialEq)]
// 	pub struct Test;
// 	impl system::Trait for Test {
// 		type Origin = Origin;
// 		type Index = u64;
// 		type BlockNumber = u64;
// 		type Hash = H256;
// 		type Hashing = BlakeTwo256;
// 		type Digest = Digest;
// 		type AccountId = u64;
// 		type Lookup = IdentityLookup<Self::AccountId>;
// 		type Header = Header;
// 		type Event = ();
// 		type Log = DigestItem;
// 	}
// 	impl Trait for Test {
// 		type Event = ();
// 	}
// 	type PRC20 = Module<Test>;

// 	// This function basically just builds a genesis storage key/value store according to
// 	// our desired mockup.
// 	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
// 		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
// 	}

// 	#[test]
// 	fn it_works_for_default_value() {
// 		with_externalities(&mut new_test_ext(), || {
// 			// Just a dummy test for the dummy funtion `do_something`
// 			// calling the `do_something` function with a value 42
// 			assert_ok!(PRC20::do_something(Origin::signed(1), 42));
// 			// asserting that the stored value is equal to what we stored
// 			assert_eq!(PRC20::something(), Some(42));
// 		});
// 	}
// }
