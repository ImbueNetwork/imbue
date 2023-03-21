use crate as pallet_briefs;
use frame_support::{
    parameter_types,
    traits::{ConstU32, Nothing},
    weights::{ConstantMultiplier, IdentityFee},
    PalletId,
};

use frame_system::EnsureRoot;
use sp_core::{sr25519::Signature, H256};

use crate::mock::sp_api_hidden_includes_construct_runtime::hidden_include::traits::GenesisBuild;
use crate::pallet::{
    BriefHash,
    BriefMilestone
};
use crate::traits::BriefEvolver;
use crate::MilestoneKey;

use common_types::CurrencyId;
use core::marker::PhantomData;
use frame_support::dispatch::EncodeLike;
use frame_support::once_cell::sync::Lazy;
use orml_traits::MultiCurrency;
use proposals::{Project, Projects};
use sp_core::sr25519;
use sp_runtime::DispatchResult;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    BuildStorage,
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::{
    convert::{TryFrom, TryInto},
    str,
    vec::Vec,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type BlockNumber = u64;
pub type Amount = i128;
pub type Balance = u64;
pub type Moment = u64;
//type AccountId = sp_core::sr25519::Public;

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Native;
}

pub type AdaptedBasicCurrency =
    orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;

impl orml_currencies::Config for Test {
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type WeightInfo = ();
}

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Storage},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>},
        BriefsMod: pallet_briefs::{Pallet, Call, Storage, Event<T>},
        Proposals: proposals::{Pallet, Call, Storage, Event<T>},
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>},
    }
);

orml_traits::parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        1
    };
}

parameter_types! {
    pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account_truncating();
    pub MaxLocks: u32 = 2;
}

impl orml_tokens::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Amount = i128;
    type CurrencyId = common_types::CurrencyId;
    type CurrencyHooks = ();
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type DustRemovalWhitelist = Nothing;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const TransactionByteFee: u64 = 1;
    pub const OperationalFeeMultiplier: u8 = 5;
}
impl pallet_transaction_payment::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type WeightToFee = IdentityFee<u64>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = ();
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type RuntimeEvent = RuntimeEvent;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

parameter_types! {
    pub const GracePeriod: u64 = 5;
    pub const UnsignedInterval: u64 = 128;
    pub const UnsignedPriority: u64 = 1 << 20;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 5;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountStore = System;
    type Balance = u64;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Config for Test {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub MinimumBounty: Balance = 10_000u32.into();
    pub MinimumDeposit: Balance = 1000u32.into();
    pub MaximumApplicants: u32 = 10_000u32;
    pub ApplicationSubmissionTime: BlockNumber = 1000u32.into();
    pub MaxBriefOwners: u32 = 100;
    pub MaxMilestones: u32 = 100;

}

impl pallet_briefs::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RMultiCurrency = Tokens;
    type BriefHasher = BlakeTwo256;
    type AuthorityOrigin = EnsureRoot<AccountId>;
    type BriefEvolver = Proposals;
    type MaxBriefOwners = MaxBriefOwners;
    type MaxMilestones = MaxMilestones;
}

parameter_types! {
    pub const TwoWeekBlockUnit: u32 = 100800u32;
    pub const ProposalsPalletId: PalletId = PalletId(*b"imbgrant");
    pub NoConfidenceTimeLimit: BlockNumber = 100800u32.into();
    pub PercentRequiredForVoteToPass: u8 = 75u8;
    pub MaximumContributorsPerProject: u32 = 5000;
    pub RefundsPerBlock: u8 = 2;
}

// Requires binding howerver they may be a more succinct way of doing this.
impl<T: proposals::Config> BriefEvolver<AccountId, Balance, BlockNumber, BriefMilestone> for proposals::Pallet<T>
where
    Project<sp_core::sr25519::Public, u64, u64, u64>: EncodeLike<
        Project<
            <T as frame_system::Config>::AccountId,
            <<T as proposals::Config>::MultiCurrency as MultiCurrency<
                <T as frame_system::Config>::AccountId,
            >>::Balance,
            <T as frame_system::Config>::BlockNumber,
            <T as pallet_timestamp::Config>::Moment,
        >,
    >,
{
    fn convert_to_proposal(
        brief_owners: Vec<AccountId>,
        bounty_total: Balance,
        currency_id: CurrencyId,
        contributions: BTreeMap<AccountId, Balance>,
        created_at: BlockNumber,
        brief_hash: BriefHash,
        applicant: AccountId,
        milestones: BTreeMap<MilestoneKey, BriefMilestone>
    ) -> Result<(), ()> {
        // todo: valicdation
        // tests:
        // lots of tests.

        let project: Project<AccountId, Balance, BlockNumber, Moment> = Project {
            milestones: BTreeMap::new(), //milestones, todo: type conversion
            contributions: BTreeMap::new(), // todo: keep track of contributions + type conversion,
            currency_id,
            required_funds: Default::default(),//todo getsum,
            withdrawn_funds: 0u32.into(),
            raised_funds: Default::default(), //todo: getsum,
            initiator: applicant,
            create_block_number: System::block_number(),
            approved_for_funding: true,
            funding_threshold_met: true,
            cancelled: false,
            agreement_hash: brief_hash,
            // Maybe we dont need this new field because we have create_block_number 
            work_started_at: Some(System::block_number()),
        };

        Projects::<T>::insert(0, project);
    

        Ok(())
    }
}

impl proposals::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ProposalsPalletId;
    type AuthorityOrigin = EnsureRoot<AccountId>;
    type MultiCurrency = Tokens;
    type WeightInfo = ();
    type MaxProjectsPerRound = ConstU32<4>;
    // Adding 2 weeks as th expiration time
    type MaxWithdrawalExpiration = TwoWeekBlockUnit;
    type NoConfidenceTimeLimit = NoConfidenceTimeLimit;
    type PercentRequiredForVoteToPass = PercentRequiredForVoteToPass;
    type MaximumContributorsPerProject = MaximumContributorsPerProject;
    type RefundsPerBlock = RefundsPerBlock;
}

parameter_types! {
    pub const BasicDeposit: u64 = 10;
    pub const FieldDeposit: u64 = 10;
    pub const SubAccountDeposit: u64 = 10;
    pub const MaxSubAccounts: u32 = 2;
    pub const MaxAdditionalFields: u32 = 2;
    pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Slashed = ();
    type BasicDeposit = BasicDeposit;
    type FieldDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type MaxAdditionalFields = MaxAdditionalFields;
    type MaxRegistrars = MaxRegistrars;
    type RegistrarOrigin = EnsureRoot<AccountId>;
    type ForceOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}

parameter_types! {
    pub const UnitWeightCost: u64 = 10;
    pub const MaxInstructions: u32 = 100;
}
pub static ALICE: Lazy<sr25519::Public> = Lazy::new(|| sr25519::Public::from_raw([125u8; 32]));
pub static BOB: Lazy<sr25519::Public> = Lazy::new(|| sr25519::Public::from_raw([126u8; 32]));
pub static CHARLIE: Lazy<sr25519::Public> = Lazy::new(|| sr25519::Public::from_raw([127u8; 32]));

pub(crate) fn build_test_externality() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    GenesisConfig::default().assimilate_storage(&mut t).unwrap();

    orml_tokens::GenesisConfig::<Test> {
        balances: {
            vec![*ALICE, *BOB, *CHARLIE]
                .into_iter()
                .map(|id| (id, CurrencyId::Native, 1000000))
                .collect::<Vec<_>>()
        },
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
