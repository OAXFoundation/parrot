//! The Substrate Node Template runtime. This can be compiled with
//! `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and
// requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use core::convert::TryFrom;
use grandpa::fg_primitives;
use grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{
        BlakeTwo256, Block as BlockT, IdentifyAccount, IdentityLookup, NumberFor, Saturating,
        Verify,
    },
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, MultiSignature, Perquintill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use balances::Call as BalancesCall;
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{Currency, FindAuthor, Imbalance, KeyOwnerProofSystem, OnUnbalanced, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        IdentityFee, Weight,
    },
    ConsensusEngineId, StorageValue,
};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};
pub use timestamp::Call as TimestampCall;

// OAX custom pallets
pub use burn;
pub use delegation;
pub use multi_transfer;
pub use prc20;

/// Implementations of some helper traits passed into runtime modules
/// as associated types.
pub mod impls;
use impls::Author;
use impls::TargetedFeeAdjustment;

/// Constant values used within the runtime.
pub mod constants;
use constants::currency::*;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction
/// signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally
/// make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than
///  4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that
/// don't need to know the specifics of the runtime. They can then be made to
/// be agnostic over specific formats of data like extrinsics, allowing for
///  them to continue syncing the network through upgrades to even the
///  core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("parrot"),
    impl_name: create_runtime_str!("parrot"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// These time units are defined in number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when
/// compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            // for fees, 80% to treasury, 20% to author
            let mut split = fees.ration(80, 20);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any, 80% to treasury, 20% to author
                // (though this can be anything)
                tips.ration_merge_into(80, 20, &mut split);
            }
            //TODO: Switch consensus to babe, or pay the block author 80%!
            // Currently all fees 80% and 20% is paid to burner
            Author::on_nonzero_unbalanced(split.0);
            Burner::on_nonzero_unbalanced(split.1);
        }
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    /// Assume 10% of weight for average on_initialize calls.
    pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
        .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

impl system::Trait for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is
    /// passed in dispatchers.
    type Lookup = IdentityLookup<AccountId>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep
    /// (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Maximum weight of each block.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// The weight of the overhead invoked on the block import process,
    /// independent of the
    /// extrinsics included in that block.
    type BlockExecutionWeight = BlockExecutionWeight;
    /// The base weight of any extrinsic processed by the runtime, independent
    /// of the logic of that extrinsic.
    ///  (Signature verification, nonce increment, fee, etc...)
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    /// The maximum weight that a single extrinsic of `Normal` dispatch
    /// class can have,
    /// idependent of the logic of that extrinsics. (Roughly max block weight
    /// - average on initialize cost).
    type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
    /// Maximum size of all encoded transactions (in bytes) that are
    /// allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Portion of the block weight that is available to all normal
    /// transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type ModuleToIndex = ModuleToIndex;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = balances::AccountData<Balance>;
}

impl aura::Trait for Runtime {
    type AuthorityId = AuraId;
}

impl grandpa::Trait for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProofSystem = ();

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type HandleEquivocation = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1 * DOLLARS;
}

impl balances::Trait for Runtime {
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
}

parameter_types! {
    pub const TransactionByteFee: Balance  = 10 * MILLICENTS;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
}

impl transaction_payment::Trait for Runtime {
    type Currency = balances::Module<Runtime>;
    type OnTransactionPayment = DealWithFees;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = TargetedFeeAdjustment<TargetBlockFullness>;
}

impl sudo::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

impl pallet_authorship::Trait for Runtime {
    // I wanted to do this, but the types were incompatible
    // type FindAuthor = aura::AuraAuthorId<Self>;

    // So instead I tried this. The adapter is defined immediately below
    type FindAuthor = AuraAccountAdapter;
    type UncleGenerations = ();
    type FilterUncle = ();
    type EventHandler = ();
}
// This struct is (supposed to be) an adapter type that converts an AuraId
// into an AccountId
pub struct AuraAccountAdapter;

impl FindAuthor<AccountId> for AuraAccountAdapter {
    fn find_author<'a, I>(digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        aura::AuraAuthorId::<Runtime>::find_author(digests)
            .and_then(|k| AccountId::try_from(k.as_ref()).ok())
    }
}

// This is a configurable constant for the multiTransfer limit (max number
//of transfers in a single multi_transfer)
parameter_types! {
    pub const MaxTransfers: u8 = 100;
}
// Implement the multi_transfer trait for runtime
impl multi_transfer::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MaxTransfers = MaxTransfers;
}

// Implement the delegation trait for runtime
impl delegation::Trait for Runtime {
    type Event = Event;
    type Public = <MultiSignature as Verify>::Signer;
    type Signature = MultiSignature;
    type Currency = Balances;
}

//Implement the prc20 trait for runtime
impl prc20::Trait for Runtime {
    type Event = Event;
    type TokenBalance = u128;
    type TokenId = u128;
    type Public = <MultiSignature as Verify>::Signer;
    type Signature = MultiSignature;
    type MaxTransfers = MaxTransfers;
}

// This is a configurable constant, that sets the number of blocks to
// run a burn (every 5 blocks)
parameter_types! {
    pub const BurnPeriod: BlockNumber = 1 * MINUTES;
}
// Implement the burn trait for runtime
impl burn::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BurnPeriod = BurnPeriod;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{Module, Call, Config, Storage, Event<T>},
        RandomnessCollectiveFlip: randomness_collective_flip::
            {Module, Call, Storage},
        Timestamp: timestamp::{Module, Call, Storage, Inherent},
        Aura: aura::{Module, Config<T>, Inherent(Timestamp)},
        Grandpa: grandpa::{Module, Call, Storage, Config, Event},
        Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: transaction_payment::{Module, Storage},
        Sudo: sudo::{Module, Call, Config<T>, Storage, Event<T>},
        Authorship: pallet_authorship::{Module, Call, Storage},
        // OAX custom pallets
        MultiTransfer: multi_transfer::{Module, Call, Event<T>},
        PRC20: prc20::{Module, Call, Storage, Event<T>},
        Delegation: delegation::{Module, Call, Event<T>},
        Burner: burn::{Module, Call, Storage, Event<T>, Config<T>},
    }
);

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    system::CheckSpecVersion<Runtime>,
    system::CheckTxVersion<Runtime>,
    system::CheckGenesis<Runtime>,
    system::CheckEra<Runtime>,
    system::CheckNonce<Runtime>,
    system::CheckWeight<Runtime>,
    transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    frame_executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic)
        -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData)
        -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::
        TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }
}
