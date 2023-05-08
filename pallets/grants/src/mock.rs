use crate as pallet_grants;
use common_types::CurrencyId;
use frame_support::once_cell::sync::Lazy;
use frame_support::traits::{ConstU16, ConstU64, Nothing};
use frame_support::{pallet_prelude::*, parameter_types, PalletId};
use frame_system::EnsureRoot;
use orml_traits::MultiCurrency;
use sp_core::sr25519::{Public, Signature};
use sp_core::H256;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;
pub type Balance = u64;
type Moment = u64;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Grant: pallet_grants,
        Tokens: orml_tokens,
        TimeStamp: pallet_timestamp,
        Proposals: pallet_proposals,
        Identity: pallet_identity,
        Balances: pallet_balances,
    }
);

parameter_types! {
    pub const ExistentialDeposit: u64 = 5;
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

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

orml_traits::parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        100
    };
}

parameter_types! {
    pub const MaxReserves: u32 = 50;
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
    pub MaxMilestonesPerGrant: u32 = 50;
    pub MaxApprovers: u32 = 100;
}

impl pallet_grants::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RMultiCurrency = Tokens;
    type MaxMilestonesPerGrant = MaxMilestonesPerGrant;
    type MaxApprovers = MaxApprovers;
    type IntoProposal = pallet_proposals::Pallet<Test>;
    type CancellingAuthority = EnsureRoot<AccountId>;
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
    pub const TwoWeekBlockUnit: u32 = 100800u32;
    pub const ProposalsPalletId: PalletId = PalletId(*b"imbgrant");
    pub NoConfidenceTimeLimit: BlockNumber = 100800u32.into();
    pub PercentRequiredForVoteToPass: u8 = 75u8;
    pub MaximumContributorsPerProject: u32 = 5000;
    pub RefundsPerBlock: u8 = 2;
    pub IsIdentityRequired: bool = false;
    pub MaxMilestonesPerProject: u32 = 50;
}

impl pallet_proposals::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ProposalsPalletId;
    type AuthorityOrigin = EnsureRoot<AccountId>;
    type MultiCurrency = Tokens;
    type WeightInfo = ();
    type MaxProjectsPerRound = ConstU32<4>;
    type MaxWithdrawalExpiration = TwoWeekBlockUnit;
    type NoConfidenceTimeLimit = NoConfidenceTimeLimit;
    type PercentRequiredForVoteToPass = PercentRequiredForVoteToPass;
    type MaximumContributorsPerProject = MaximumContributorsPerProject;
    type RefundsPerBlock = RefundsPerBlock;
    type IsIdentityRequired = IsIdentityRequired;
    type MilestoneVotingWindow = TwoWeekBlockUnit;
    type RefundHandler = pallet_proposals::traits::MockRefundHandler<Test>;
    type MaxMilestonesPerProject = MaxMilestonesPerProject;
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

pub static ALICE: Lazy<Public> = Lazy::new(|| Public::from_raw([125u8; 32]));
pub static BOB: Lazy<Public> = Lazy::new(|| Public::from_raw([126u8; 32]));
pub static CHARLIE: Lazy<Public> = Lazy::new(|| Public::from_raw([127u8; 32]));
pub fn gen_grant_id(seed: u8) -> pallet_grants::GrantId {
    H256::from([seed; 32])
}
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        let initial_balance = 10_000_000u64;
        System::set_block_number(1);
        let _ = Tokens::deposit(CurrencyId::Native, &ALICE, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &BOB, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &CHARLIE, initial_balance);
    });
    ext
}
