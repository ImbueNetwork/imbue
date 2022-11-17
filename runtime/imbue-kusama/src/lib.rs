#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;

use pallet_collective::EnsureProportionAtLeast;
use sp_runtime::{
    create_runtime_str, generic,
    generic::Era,
    impl_opaque_keys,
    traits::{
        AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto,
        Extrinsic as ExtrinsicT, Verify,
    },
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Perbill, Permill,
};
use sp_std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    prelude::*,
};

use crate::xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// Weights used in the runtime

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, ensure, parameter_types,
    traits::{
        fungibles, Contains, Currency as PalletCurrency, EnsureOriginWithArg, EqualPrivilegeOnly,
        Everything, Get, Imbalance, IsInVec, Nothing, OnUnbalanced, Randomness,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        ConstantMultiplier, DispatchClass, IdentityFee, Weight,
    },
    PalletId, StorageValue,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use orml_currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;

use crate::sp_api_hidden_includes_IMPL_RUNTIME_APIS::sp_api::Encode;
pub use common_runtime::{
    asset_registry::{AuthorityOrigin, CustomAssetProcessor},
    common_xcm::{general_key, FixedConversionRateProvider},
    xcm_fees::{default_per_second, ksm_per_second, native_per_second, WeightToFee},
};
pub use common_types::{CurrencyId, CustomMetadata};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use polkadot_runtime_common::SlowAdjustingFeeUpdate;

use xcm_executor::XcmExecutor;

// XCM imports
pub use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};

use common_runtime::currency::*;
pub use common_runtime::Index;
pub use xcm::latest::prelude::*;

/// common types for the runtime.
pub use common_runtime::*;

mod weights;

pub mod xcm_config;

pub use crate::xcm_config::*;

pub type SessionHandlers = ();

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("imbue"),
    impl_name: create_runtime_str!("imbue"),
    authoring_version: 2,
    spec_version: 1032,
    impl_version: 2,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 2,
    state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
/// We allow for .5 seconds of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND.saturating_div(2);

pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const Version: RuntimeVersion = VERSION;
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = common_runtime::Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    type Hash = Hash;

    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Runtime version.
    type Version = Version;
    /// Converts a module to an index of this module in the runtime.
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = frame_support::traits::Everything;
    type SystemWeightInfo = ();
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type SS58Prefix = SS58Prefix;
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

type NegativeImbalance = <Balances as PalletCurrency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(mut fees) = fees_then_tips.next() {
            if let Some(tips) = fees_then_tips.next() {
                tips.merge_into(&mut fees);
            }
            // for fees and tips, 100% to treasury
            Treasury::on_unbalanced(fees);
        }
    }
}

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const NativeTokenTransferFee: u128 = NATIVE_TOKEN_TRANSFER_FEE;
    pub const CreationFee: Balance = 1 * MICRO_IMBU;
    pub const TransactionByteFee: Balance = MICRO_IMBU * 10;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// Handler for the unbalanced reduction when removing a dust account.
    type DustRemoval = Treasury;
    /// The overarching event type.
    type Event = Event;
    /// The minimum amount required to keep an account open.
    type ExistentialDeposit = NativeTokenTransferFee;
    /// The means of storing the balances of an account.
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Self>;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

impl pallet_randomness_collective_flip::Config for Runtime {}

parameter_types! {
    pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
    type Event = Event;
    type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
    type WeightToFee = WeightToFee;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnSystemEvent = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

/// XCMP Queue is responsible to handle XCM messages coming directly from sibling parachains.
impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = PolkadotXcm;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
}

parameter_types! {
    pub const MinVestedTransfer: Balance = MIN_VESTING * IMBU;
}

impl pallet_vesting::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BlockNumberToBalance = ConvertInto;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = pallet_vesting::weights::SubstrateWeight<Self>;
    const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
    pub const MinimumReward: Balance = 0;
    pub const Initialized: bool = false;
    pub const InitializationPayment: Perbill = Perbill::from_percent(30);
    pub const MaxInitContributorsBatchSizes: u32 = 500;
    pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
    type PalletsOrigin = OriginCaller;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = NORMAL_DISPATCH_RATIO * RuntimeBlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl orml_unknown_tokens::Config for Runtime {
    type Event = Event;
}

pub struct AlwaysPrivilege;
impl<T: Sized> frame_support::traits::PrivilegeCmp<T> for AlwaysPrivilege {
    fn cmp_privilege(_: &T, _: &T) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

parameter_types! {
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = deposit(2, 64);
    pub const PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = ();
    type Event = Event;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type MaxSize = PreimageMaxSize;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Self>;
    type PreimageProvider = Preimage;
    type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type Event = Event;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = ();
}

parameter_types! {
    /// The maximum amount of time (in blocks) for council members to vote on motions.
    /// Motions may end in fewer blocks if enough votes are cast to determine the result.
    pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
    /// The maximum number of Proposlas that can be open in the council at once.
    pub const CouncilMaxProposals: u32 = 100;
    /// The maximum number of council members.
    pub const CouncilMaxMembers: u32 = 100;

    /// The maximum amount of time (in blocks) for technical committee members to vote on motions.
    /// Motions may end in fewer blocks if enough votes are cast to determine the result.
    pub const TechCommitteeMotionDuration: BlockNumber = 3 * DAYS;
    /// The maximum number of Proposlas that can be open in the technical committee at once.
    pub const TechCommitteeMaxProposals: u32 = 100;
    /// The maximum number of technical committee members.
    pub const TechCommitteeMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
type TechnicalCollective = pallet_collective::Instance2;

impl pallet_collective::Config<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Event = Event;
    type Proposal = Call;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_collective::Config<TechnicalCollective> for Runtime {
    type Origin = Origin;
    type Event = Event;
    type Proposal = Call;
    type MotionDuration = TechCommitteeMotionDuration;
    type MaxProposals = TechCommitteeMaxProposals;
    type MaxMembers = TechCommitteeMaxMembers;
    type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
    type AddOrigin = MoreThanHalfCouncil;
    type Event = Event;
    type MaxMembers = CouncilMaxMembers;
    type MembershipChanged = Council;
    type MembershipInitialized = Council;
    type PrimeOrigin = MoreThanHalfCouncil;
    type RemoveOrigin = MoreThanHalfCouncil;
    type ResetOrigin = MoreThanHalfCouncil;
    type SwapOrigin = MoreThanHalfCouncil;
    type WeightInfo = ();
}

impl pallet_membership::Config<pallet_membership::Instance2> for Runtime {
    type AddOrigin = MoreThanHalfCouncil;
    type Event = Event;
    type MaxMembers = TechCommitteeMaxMembers;
    type MembershipChanged = TechnicalCommittee;
    type MembershipInitialized = TechnicalCommittee;
    type PrimeOrigin = MoreThanHalfCouncil;
    type RemoveOrigin = MoreThanHalfCouncil;
    type ResetOrigin = MoreThanHalfCouncil;
    type SwapOrigin = MoreThanHalfCouncil;
    type WeightInfo = ();
}

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 7 * DAYS;
    pub const VotingPeriod: BlockNumber = 7 * DAYS;
    pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
    pub MinimumDeposit: Balance = 500 * IMBU;
    pub const EnactmentPeriod: BlockNumber = 2 * DAYS;
    pub const CooloffPeriod: BlockNumber = 7 * DAYS;
    pub const InstantAllowed: bool = true;
    pub const MaxVotes: u32 = 100;
    pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
    type BlacklistOrigin = EnsureRoot<AccountId>;
    // To cancel a proposal before it has been passed, the technical committee must be unanimous or
    // Root must agree.
    type CancelProposalOrigin = HalfOfCouncil;

    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin = HalfOfCouncil;
    type CooloffPeriod = CooloffPeriod;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type Event = Event;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin = HalfOfCouncil;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin = HalfOfCouncil;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin = HalfOfCouncil;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin = HalfOfCouncil;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    type InstantAllowed = InstantAllowed;
    type InstantOrigin = HalfOfCouncil;
    type LaunchPeriod = LaunchPeriod;
    type MaxProposals = MaxProposals;
    type MaxVotes = MaxVotes;
    type MinimumDeposit = MinimumDeposit;
    type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
    type PalletsOrigin = OriginCaller;
    type PreimageByteDeposit = PreimageByteDeposit;
    type Proposal = Call;
    type Scheduler = Scheduler;
    type Slash = Treasury;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cool-off period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
    type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
    type VotingPeriod = VotingPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 5;
}

// We only use find_author to pay in anchor pallet
impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = (CollatorSelection,);
}

parameter_types! {
    pub const Period: u32 = 6 * HOURS;
    pub const Offset: u32 = 0;
}

pub struct ValidatorOf;
impl<T> sp_runtime::traits::Convert<T, Option<T>> for ValidatorOf {
    fn convert(t: T) -> Option<T> {
        Some(t)
    }
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = CollatorSelection;
    // Essentially just Aura, but lets be pedantic.
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Self>;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as Verify>::Signer,
        account: AccountId,
        nonce: <Runtime as frame_system::Config>::Index,
    ) -> Option<(Call, <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload)> {
        use sp_runtime::traits::StaticLookup;
        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;

        let current_block = System::block_number()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1)
            .into();
        let tip = 0;
        let era = Era::mortal(period, current_block);
        let extra = (
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(era),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                log::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;

        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let (call, extra, _) = raw_payload.deconstruct();
        let address = <Runtime as frame_system::Config>::Lookup::unlookup(account);
        Some((call, (address, signature, extra)))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as sp_runtime::traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = UncheckedExtrinsic;
}

/// All council members must vote yes to create this origin.
type HalfOfCouncil = EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
/// A majority of the Unit body from Rococo over XCM is our required administration origin.
pub type AdminOrigin = EnsureRootOr<HalfOfCouncil>;
pub type MoreThanHalfCouncil = EnsureRootOr<HalfOfCouncil>;

// pub type MoreThanHalfCouncil = EnsureOneOf<
// 	EnsureRoot<AccountId>,
// 	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
// >;

// Parameterize collator selection pallet
parameter_types! {
    pub const PotId: PalletId = PalletId(*b"PotStake");
    pub const MaxCandidates: u32 = 1000;
    pub const MinCandidates: u32 = 5;
    pub const SessionLength: BlockNumber = 6 * HOURS;
    pub const MaxInvulnerables: u32 = 100;
}

// Implement Collator Selection pallet configuration trait for the runtime
impl pallet_collator_selection::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type UpdateOrigin = MoreThanHalfCouncil;
    type PotId = PotId;
    type MaxCandidates = MaxCandidates;
    type MinCandidates = MinCandidates;
    type MaxInvulnerables = MaxInvulnerables;
    // should be a multiple of session or things will get inconsistent
    type KickThreshold = Period;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ValidatorRegistration = Session;
    type WeightInfo = pallet_collator_selection::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const AssetDeposit: Balance = 1 * IMBU;
    pub const AssetAccountDeposit: Balance = 1 * IMBU;
    pub const ApprovalDeposit: Balance = 100 * MILLI_IMBU;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 1 * IMBU;
    pub const MetadataDepositPerByte: Balance = 10 * MILLI_IMBU;
    pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        // every currency has a zero existential deposit
        match currency_id {
            _ => 0,
        }
    };
}

parameter_types! {
    pub ORMLMaxLocks: u32 = 2;
    pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

impl orml_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = IBalance;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = orml_tokens::TransferDust<Runtime, TreasuryAccount>;
    type MaxLocks = ORMLMaxLocks;
    type DustRemovalWhitelist = Nothing;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type OnNewTokenAccount = ();
    type OnKilledTokenAccount = ();
}

impl orml_asset_registry::Config for Runtime {
    type AssetId = CurrencyId;
    type AssetProcessor = asset_registry::CustomAssetProcessor;
    type AuthorityOrigin = asset_registry::AuthorityOrigin<Origin, EnsureRootOr<HalfOfCouncil>>;
    type Balance = Balance;
    type CustomMetadata = CustomMetadata;
    type Event = Event;
    type WeightInfo = ();
}

pub type Amount = i128;

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Native;
}

impl orml_currencies::Config for Runtime {
    type MultiCurrency = OrmlTokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    // Add one item in storage and take 258 bytes
    pub const BasicDeposit: Balance = currency::deposit(1, 258);
    // Not add any item to the storage but takes 66 bytes
    pub const FieldDeposit: Balance = currency::deposit(0, 66);
    // Add one item in storage and take 53 bytes
    pub const SubAccountDeposit: Balance = currency::deposit(1, 53);
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 100;
    pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BasicDeposit = BasicDeposit;
    type FieldDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type MaxAdditionalFields = MaxAdditionalFields;
    type MaxRegistrars = MaxRegistrars;
    type Slashed = Treasury;
    type ForceOrigin = EnsureRootOr<HalfOfCouncil>;
    type RegistrarOrigin = EnsureRootOr<HalfOfCouncil>;
    type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ProposalsPalletId: PalletId = PalletId(*b"imbgrant");
    pub const MaxProjectsPerRound: u32 = 256;
    pub const MaxWithdrawalExpiration: BlockNumber = 180 * DAYS;
    pub const NoConfidenceTimeLimit: BlockNumber = 14 * DAYS;
    pub const PercentRequiredForVoteToPass: u8 = 75;
}

parameter_types! {
    // 5% of the proposal value need to be bonded. This will be returned
    pub const ProposalBond: Permill = Permill::from_percent(5);

    // Minimum amount to bond per proposal. This will be the least that gets bonded per proposal
    // if the above yields to lower value
    pub const ProposalBondMinimum: Balance = 100 * IMBU;

    // Maximum amount to bond per proposal. This will be the most that gets bonded per proposal
    pub const ProposalBondMaximum: Balance = 500 * IMBU;

    // periods between treasury spends
    pub const SpendPeriod: BlockNumber = 30 * DAYS;

    // percentage of treasury we burn per Spend period if there is a surplus
    // If the treasury is able to spend on all the approved proposals and didn't miss any
    // then we burn % amount of remaining balance
    // If the treasury couldn't spend on all the approved proposals, then we dont burn any
    pub const Burn: Permill = Permill::from_percent(1);

    // treasury pallet account id
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");

    // Maximum number of approvals that can be in the spending queue
    pub const MaxApprovals: u32 = 100;

    }

impl pallet_treasury::Config for Runtime {
    type Currency = Balances;
    // either democracy or 75% of council votes
    type ApproveOrigin = MoreThanHalfCouncil;
    type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
    // either democracy or more than 50% council votes
    type RejectOrigin = EnsureRootOr<HalfOfCouncil>;
    type Event = Event;
    // slashed amount goes to treasury account
    type OnSlash = Treasury;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalBondMaximum = ProposalBondMaximum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type PalletId = TreasuryPalletId;
    // we burn and dont handle the unbalance
    type BurnDestination = ();
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Self>;
    type SpendFunds = ();
    type MaxApprovals = MaxApprovals;
}
parameter_types! {
    // Percent fee taken when a project is approved
    pub const PercentFeeOnApproval: u8 = 3;
}

impl proposals::Config for Runtime {
    type Event = Event;
    type PalletId = ProposalsPalletId;
    type MultiCurrency = Currencies;
    type AuthorityOrigin = AdminOrigin;
    type MaxProjectsPerRound = MaxProjectsPerRound;
    type MaxWithdrawalExpiration = MaxWithdrawalExpiration;
    type NoConfidenceTimeLimit = NoConfidenceTimeLimit;
    type PercentRequiredForVoteToPass = PercentRequiredForVoteToPass;
    type WeightInfo = ();
    type TreasuryId = TreasuryAccount;
    type PercentFeeOnApproval = PercentFeeOnApproval;

}

construct_runtime! {
    pub enum Runtime where
        Block = Block,
        NodeBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 1,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
        Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 4,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 5,
        Treasury: pallet_treasury::{Pallet, Storage, Config, Event<T>, Call} = 6,
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 7,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 8,
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 9,
        Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,

        CouncilMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 11,
        TechnicalMembership: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>} = 12,

        CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 13,
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 14,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 15,

        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 16,

        ParachainSystem: cumulus_pallet_parachain_system::{	Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,} = 17,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 18,

        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 19,
        Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 20,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>}  = 21,
        Utility: pallet_utility::{Pallet, Call, Event} = 22,
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 23,

        Aura: pallet_aura::{Pallet, Config<T>} = 24,
        AuraExt: cumulus_pallet_aura_ext::{Pallet, Config} = 25,

        // XCM helpers.
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 26,
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 27,
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin} = 28,
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 29,
        
        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 30,

        Currencies: orml_currencies::{Pallet, Call} = 31,
        OrmlAssetRegistry: orml_asset_registry::{Pallet, Storage, Call, Event<T>, Config<T>} = 32,
        OrmlTokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>} = 33,
        OrmlXcm: orml_xcm::{Pallet, Call, Event<T>} = 34,

        UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 35,



        // Imbue Pallets
        ImbueProposals: proposals::{Pallet, Call, Storage, Event<T>} = 100,
    }
}

pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    (),
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_timestamp, Timestamp]
        [proposals, ImbueProposals]
    );
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    use sp_runtime::{generic, traits::BlakeTwo256};

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
    }
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(
            extrinsic: <Block as BlockT>::Extrinsic,
        ) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }

        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info(header)
        }
    }


    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            return (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            impl frame_system_benchmarking::Config for Runtime {}
            impl baseline::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);

            Ok(batches)
        }
    }



}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
    fn check_inherents(
        block: &Block,
        relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
    ) -> sp_inherents::CheckInherentsResult {
        let relay_chain_slot = relay_state_proof
            .read_slot()
            .expect("Could not read the relay chain slot from the proof");

        let inherent_data =
            cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
                relay_chain_slot,
                sp_std::time::Duration::from_secs(6),
            )
            .create_inherent_data()
            .expect("Could not create the timestamp inherent data");

        inherent_data.check_extrinsics(&block)
    }
}

cumulus_pallet_parachain_system::register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
    CheckInherents = CheckInherents,
}
