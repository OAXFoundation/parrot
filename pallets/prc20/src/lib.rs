#![cfg_attr(not(feature = "std"), no_std)]

/// A runtime module for doing multi-transfers!



use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult, ensure, Parameter, traits::{Currency, ExistenceRequirement}};

// the decl_event macros expands to look for system, and its actually called frame_system here so this line makes life easier
use frame_system as system;
use system::ensure_signed;
use frame_support::weights::SimpleDispatchInfo;
use codec::{Codec, Decode, Encode, HasCompact};
use sp_std::vec::Vec;
use sp_std::{ convert::{TryInto}};
//use crate::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::Zero;
use sp_std::if_std;
use sp_runtime::traits::{
  CheckedAdd, CheckedSub, IdentifyAccount, Member, One, StaticLookup, Verify,
  Zero,
};

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + pallet_balances::Trait{
    /// The overarching event type.

    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type TokenBalance: Parameter + Member + Codec + Default + Copy + CheckedAdd + CheckedSub;
    type TokenId: Parameter + Member + Codec + Default + Copy + CheckedAdd + CheckedSub;
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
  pub nonce: u128,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct SignedOffer<Signature, AccountId, TokenBalance, TokenId> {
  pub offer: Offer<TokenBalance, TokenId>,
  pub signature: Signature,
  pub signer: AccountId,
}
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct TransferTokenDetails< AccountId, TokenBalance> {
  pub amount: TokenBalance,
  pub to: AccountId,
}
// This is used to encode each transfer in a multiTransfer
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, Debug)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct TransferDetails< AccountId, Balance: HasCompact> {
    pub amount: Balance,
    pub to: AccountId,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as PRC20 {
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
	pub enum Event<T> where
	AccountId = <T as frame_system::Trait>::AccountId,
    TokenId = <T as Trait>::TokenId,
    TokenBalance = <T as Trait>::TokenBalance,
    Balance = BalanceOf<T>,
    {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit function, from our runtime functions
        // event for a new token creation
        NewToken(TokenId, AccountId, TokenBalance),
        // event for a simple token transfer
        Transfer(TokenId, AccountId, AccountId, TokenBalance),
        // event for approval
        Approval(TokenId, AccountId, AccountId, TokenBalance),
        // event for Swap
        Swap(TokenId,TokenBalance,TokenId,TokenBalance,AccountId,AccountId),
		SomethingStored(u32, AccountId, Balance),
		MultiTransfer(Vec<(AccountId, Balance, bool)>),
	}
);

impl<T: Trait> Module<T> {
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
    let maker_nonce: u128 = TryInto::<u128>::try_into(<system::Module<T>>::account_nonce(&signed_offer.signer)).map_err(|_| "error")?;
    // let taker_nonce: u128 =  TryInto::<u128>::try_into(<system::Module<T>>::account_nonce(&sender)).map_err(|_| "error")?;
    if_std! {println!("Maker Nonce: {:?}", maker_nonce.clone());
            //  println!("Taker Nonce: {:?}", taker_nonce);
             println!("Signed Offer Nonce: {:?}", signed_offer.offer.nonce.clone());
            }
    // ensure maker nonce is correct (replay protection)
    ensure!(   maker_nonce == signed_offer.offer.nonce , "Nonce is incorrect!");
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