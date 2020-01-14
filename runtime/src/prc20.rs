/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
use frame_support::{decl_event, decl_module, decl_storage, dispatch, ensure, Parameter};
use sp_runtime::traits::{
  CheckedAdd, CheckedSub, IdentifyAccount, Member, One, SimpleArithmetic, StaticLookup, Verify,
  Zero,
};
use system::ensure_signed;
//use system::inc_account_nonce;

use codec::{Codec, Decode, Encode};
use sp_io::misc::print_utf8;
/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait {
  // TODO: Add other types and constants required configure this module.

  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
  type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy;
  type TokenId: Parameter + Member + SimpleArithmetic + Codec + Default + Copy;

  type Public: IdentifyAccount<AccountId = Self::AccountId>;
  type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
}
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct Offer<TokenBalance, TokenId> {
  pub offer_token: TokenId,
  pub offer_amount: TokenBalance,
  pub requested_token: TokenId,
  pub requested_amount: TokenBalance,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct SignedOffer<Signature, AccountId, TokenBalance, TokenId> {
  pub offer: Offer<TokenBalance, TokenId>,
  pub signature: Signature,
  pub signer: AccountId,
}

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as PRC20Module {
    TotalSupply get(total_supply): map T::TokenId => T::TokenBalance;
    Balances get(balance_of): map (T::TokenId, T::AccountId) => T::TokenBalance;
    Allowance get(allowance_of): map (T::TokenId, T::AccountId, T::AccountId) => T::TokenBalance;
    TokenCount get(token_count): T::TokenId;
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;


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
      Self::deposit_event(RawEvent::NewToken(current_id, sender, total_supply));

    }
        // do transfers like erc20 ( TokenId, To, Amount)
    fn transfer(origin,
      to: <T::Lookup as StaticLookup>::Source,
      #[compact] id: T::TokenId,
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
      spender: <T::Lookup as StaticLookup>::Source,
      #[compact] id: T::TokenId,
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
          from: T::AccountId,
          to: T::AccountId,
            #[compact] id: T::TokenId,
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

    fn swap(origin, signed_offer:  SignedOffer<T::Signature,T::AccountId, T::TokenBalance, T::TokenId>
    )	{
       let sender = ensure_signed(origin)?;

       // Ensure that the SignedOffer is signed correctly
      if Self::verify_offer_signature(signed_offer.clone()).is_ok(){
        print_utf8(b"Signature is a match! Lets trade!");
        // Ensure that offer amount is non zero
        ensure!(!signed_offer.offer.offer_amount.is_zero(), "Offer amount should be non-zero");
        // Ensure that requested amount is non zero
        ensure!(!signed_offer.offer.requested_amount.is_zero(), "Requested amount should be non-zero");
        // Make the Swap
        Self::make_swap(sender, signed_offer)?;

      };


       }


  }
}

decl_event!(
  pub enum Event<T>
  where
    AccountId = <T as system::Trait>::AccountId,
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
      AccountId
    ),
  }
);

impl<T: Trait> Module<T> {
  fn make_transfer(
    id: T::TokenId,
    from: T::AccountId,
    to: T::AccountId,
    amount: T::TokenBalance,
  ) -> dispatch::DispatchResult {
    // get balance of account
    let from_balance = Self::balance_of((id, from.clone()));
    // ensure user has enough tokens
    ensure!(from_balance >= amount, "user does not have enough tokens");
    // modify sender and receiver balance map
    <Balances<T>>::insert((id, from.clone()), from_balance - amount);
    <Balances<T>>::mutate((id, to.clone()), |balance| *balance += amount);
    // broadcast a transfer event
    Self::deposit_event(RawEvent::Transfer(id, from, to, amount));

    Ok(())
  }

  fn make_swap(
    sender: T::AccountId,
    signed_offer: SignedOffer<T::Signature, T::AccountId, T::TokenBalance, T::TokenId>,
  ) -> dispatch::DispatchResult {
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
    // prc20::inc_account_nonce(signed_offer.signer.clone());
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
  //this causes clear_on_drop error	use sp_keyring::AccountKeyring;
  use frame_support::{
    assert_noop, assert_ok, dispatch, impl_outer_origin, parameter_types, weights::Weight,
  };
  use sp_core::H256;
  // use sp_io::misc::print_utf8;
  // use sp_runtime::AccountId32;
  use node_primitives::{AccountId, Signature};
  use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    MultiSignature, Perbill,
  };
  use test_client::{self, AccountKeyring};
  impl_outer_origin! {
    pub enum Origin for TestRuntime {}
  }

  // For testing the module, we construct most of a mock runtime. This means
  // first constructing a configuration type (`Test`) which `impl`s each of the
  // configuration traits of modules we want to use.
  #[derive(Clone, Eq, PartialEq)]
  pub struct TestRuntime;
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
  }

  impl Trait for TestRuntime {
    type Event = ();
    type TokenBalance = u128;
    type TokenId = u128;
    type Public = <MultiSignature as Verify>::Signer;
    type Signature = MultiSignature;
  }

  pub type PRC20Module = Module<TestRuntime>;
  pub struct ExtBuilder;

  parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
  }

  impl balances::Trait for TestRuntime {
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
  pub type Balances = balances::Module<TestRuntime>;
  type AccountPublic = <Signature as Verify>::Signer;
  impl ExtBuilder {
    pub fn build() -> sp_io::TestExternalities {
      let mut t = system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();

      //			let mut storage = system::GenesisConfig::default()
      //                .build_storage::<Runtime>()
      //				.unwrap();

      balances::GenesisConfig::<TestRuntime> {
        balances: vec![],
        vesting: vec![],
      }
      .assimilate_storage(&mut t)
      .unwrap();
      t.into()
      // WC not sure what araa was doing below so I copied t.into() from Acala
      // runtime_io::TestExternalities::from(storage)
    }
  }

  #[test]
  fn initial_token_count_is_zero() {
    ExtBuilder::build().execute_with(|| {
      assert_eq!(PRC20Module::token_count(), 0);
    });
  }
  #[test]
  fn create_token_works() {
    ExtBuilder::build().execute_with(|| {
      let alice = AccountId::from(AccountKeyring::Alice);
      // Make sure token count is 0
      assert_eq!(PRC20Module::token_count(), 0);
      // Create Token
      assert_ok!(PRC20Module::create_token(
        Origin::signed(alice.clone()),
        10000
      ));
      // Make sure token count is now 1
      assert_eq!(PRC20Module::token_count(), 1);
      // Make sure the creator has 10000 tokens
      assert_eq!(PRC20Module::balance_of((0, alice)), 10000);
    });
  }

  #[test]
  fn transfer_token_works() {
    ExtBuilder::build().execute_with(|| {
      let alice = AccountId::from(AccountKeyring::Alice);
      let bob = AccountId::from(AccountKeyring::Bob);
      // Create Token
      assert_ok!(PRC20Module::create_token(
        Origin::signed(alice.clone()),
        10000
      ));
      // Make sure the creator has 10000 tokens
      assert_eq!(PRC20Module::balance_of((0, alice.clone())), 10000);
      // Make sure reciever has 0 tokens
      assert_eq!(PRC20Module::balance_of((0, bob.clone())), 0);
      // make sure transfer goes ok and transfer
      assert_ok!(PRC20Module::transfer(
        Origin::signed(alice.clone()),
        bob.clone(),
        0,
        10
      ));
      // make sure reciever balance is 10
      assert_eq!(PRC20Module::balance_of((0, bob)), 10);
    });
  }
  // #[test]
  // fn transfer_fails_with_no_balance(){
  //   let alice = AccountId::from(AccountKeyring::Alice);

  // 	// Create Token
  // 	assert_ok!(PRC20Module::create_token(Origin::signed(alice.clone()), 10000));
  // 	// Fails cuz 1 does not have any tokens
  // 	assert_noop!(PRC20Module::transfer(Origin::signed(alice), 0, 0, 10), "user does not have enough tokens");
  // }

  #[test]
  fn approve_token_works() {
    ExtBuilder::build().execute_with(|| {
      let alice = AccountId::from(AccountKeyring::Alice);
      let bob = AccountId::from(AccountKeyring::Bob);
      // Make sure token count is 0
      assert_eq!(PRC20Module::token_count(), 0);
      // Create Token
      assert_ok!(PRC20Module::create_token(
        Origin::signed(alice.clone()),
        10000
      ));
      // Make sure token count is now 1
      assert_eq!(PRC20Module::token_count(), 1);
      // Make sure the creator has 10000 tokens
      assert_eq!(PRC20Module::balance_of((0, alice.clone())), 10000);

      // Maybe make sure initial approval is 0
      assert_eq!(
        PRC20Module::allowance_of((0, alice.clone(), bob.clone())),
        0
      );
      // Approve account 1 to use 10 tokens from account 0
      assert_ok!(PRC20Module::approve(
        Origin::signed(alice.clone()),
        bob.clone(),
        0,
        10
      ));
      // Maybe make sure current approval is 10
      assert_eq!(PRC20Module::allowance_of((0, alice, bob)), 10);
    });
  }

  #[test]
  fn transfer_from_works() {
    ExtBuilder::build().execute_with(|| {
      let alice = AccountId::from(AccountKeyring::Alice);
      let bob = AccountId::from(AccountKeyring::Bob);
      // Make sure token count is 0
      assert_eq!(PRC20Module::token_count(), 0);
      // Create Token
      assert_ok!(PRC20Module::create_token(
        Origin::signed(alice.clone()),
        10000
      ));
      // Make sure token count is now 1
      assert_eq!(PRC20Module::token_count(), 1);
      // Make sure the creator has 10000 tokens
      assert_eq!(PRC20Module::balance_of((0, alice.clone())), 10000);
      // Maybe make sure initial approval is 0
      assert_eq!(
        PRC20Module::allowance_of((0, alice.clone(), bob.clone())),
        0
      );
      // Approve account 1 to use 10 tokens from account 0
      assert_ok!(PRC20Module::approve(
        Origin::signed(alice.clone()),
        bob.clone(),
        0,
        10
      ));
      // Maybe make sure current approval is 10
      assert_eq!(
        PRC20Module::allowance_of((0, alice.clone(), bob.clone())),
        10
      );
      // Now lets use transfer_from
      assert_ok!(PRC20Module::transfer_from(
        Origin::signed(bob.clone()),
        alice.clone(),
        bob.clone(),
        0,
        10
      ));
      // Make sure balance is updated correctly
      assert_eq!(PRC20Module::balance_of((0, bob)), 10);
    });
  }
}
