use crate as pallet_proposals;
use frame_support::{
    parameter_types,
    traits::{ConstU32, Nothing},
    PalletId,
};

use crate::*;
use common_types::CurrencyId;
use frame_system::EnsureRoot;
use sp_core::H256;

use orml_traits::MultiCurrency;
use sp_arithmetic::per_things::Percent;
use sp_runtime::{
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use sp_std::{
    convert::{TryFrom, TryInto},
    str,
};

type Block = frame_system::mocking::MockBlock<Test>;

pub type BlockNumber = u64;
pub type Amount = i128;
pub type Balance = u128;
pub type Moment = u64;
pub type AccountId = u128;

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
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Storage},
        Proposals: pallet_proposals::{Pallet, Call, Storage, Event<T>},
        IdentityPallet: pallet_identity::{Pallet, Call, Storage, Event<T>},
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
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Block = Block;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
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

parameter_types! {
    pub const GracePeriod: u64 = 5;
    pub const UnsignedInterval: u64 = 128;
    pub const UnsignedPriority: u64 = 1 << 20;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 5;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
    type RuntimeHoldReason = ();
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
    pub const ProposalsPalletId: PalletId = PalletId(*b"imbgrant");
    pub PercentRequiredForVoteToPass: Percent = Percent::from_percent(75u8);
    pub MaximumContributorsPerProject: u32 = 50;
    pub RefundsPerBlock: u8 = 2;
    pub IsIdentityRequired: bool = false;
    pub MilestoneVotingWindow: BlockNumber  =  100800u64;
    pub MaxMilestonesPerProject: u32 = 10;
    pub ImbueFee: Percent = Percent::from_percent(5u8);
    pub ExpiringProjectRoundsPerBlock: u32 = 10;
    pub ProjectStorageItem: StorageItems = StorageItems::Project;
    pub MaxProjectsPerAccount: u16 = 50;
    pub MaxJuryMembers: u32 = 100;
    pub ImbueFeeAccount: AccountId = TREASURY;
}

impl pallet_proposals::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ProposalsPalletId;
    type MultiCurrency = Tokens;
    type WeightInfo = pallet_proposals::WeightInfo<Self>;
    type PercentRequiredForVoteToPass = PercentRequiredForVoteToPass;
    type MaximumContributorsPerProject = MaximumContributorsPerProject;
    type MilestoneVotingWindow = MilestoneVotingWindow;
    type ExternalRefundHandler = pallet_proposals::traits::MockRefundHandler<Test>;
    type MaxMilestonesPerProject = MaxMilestonesPerProject;
    type ImbueFee = ImbueFee;
    type ImbueFeeAccount = ImbueFeeAccount;
    type ExpiringProjectRoundsPerBlock = ExpiringProjectRoundsPerBlock;
    type DepositHandler = MockDepositHandler;
    type ProjectStorageItem = ProjectStorageItem;
    type MaxProjectsPerAccount = MaxProjectsPerAccount;
    type DisputeRaiser = MockDisputeRaiser;
    type JurySelector = MockJurySelector;
    type AssetSignerOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
    pub const BasicDeposit: Balance = 10;
    pub const FieldDeposit: Balance = 10;
    pub const SubAccountDeposit: Balance = 10;
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
    pub MaxCandidatesPerShortlist: u32 = 100;
    pub ShortlistPeriod: BlockNumber = 100;
    pub MembershipDeposit: Balance = 50_000_000;
    pub SlashAccount: AccountId = TREASURY;
    pub DepositCurrencyId: CurrencyId = CurrencyId::Native;
}

parameter_types! {
    pub const UnitWeightCost: u64 = 10;
    pub const MaxInstructions: u32 = 100;
}
pub static ALICE: AccountId = 125;
pub static BOB: AccountId = 126;
pub static CHARLIE: AccountId = 127;
pub static DAVE: AccountId = 128;
pub static TREASURY: AccountId = 222;
pub static JOHN: AccountId = 255;

pub static JURY_1: AccountId = 1000;
pub static JURY_2: AccountId = 1001;

pub(crate) fn build_test_externality() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        let initial_balance = 100_000_000_000u128;
        System::set_block_number(1);
        let _ = Tokens::deposit(CurrencyId::Native, &ALICE, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &BOB, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &CHARLIE, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &DAVE, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &JOHN, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &TREASURY, initial_balance);
    });
    ext
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo, Copy)]
pub enum StorageItems {
    Project,
}

pub struct MockDepositHandler;
impl DepositHandler<Balance, AccountId> for MockDepositHandler {
    type DepositId = u64;
    type StorageItem = StorageItems;
    fn take_deposit(
        _who: AccountId,
        _storage_item: Self::StorageItem,
        _currency_id: CurrencyId,
    ) -> Result<Self::DepositId, DispatchError> {
        Ok(0u64)
    }
    fn return_deposit(_deposit_id: Self::DepositId) -> DispatchResult {
        Ok(())
    }
    fn slash_reserve_deposit(_deposit_id: Self::DepositId) -> DispatchResult {
        Ok(())
    }
}

pub struct MockDisputeRaiser;
impl DisputeRaiser<AccountId> for MockDisputeRaiser {
    type DisputeKey = ProjectKey;
    type SpecificId = MilestoneKey;
    type MaxJurySize = MaxJuryMembers;
    type MaxSpecifics = MaxMilestonesPerProject;
    fn raise_dispute(
        _dispute_key: Self::DisputeKey,
        _raised_by: AccountId,
        _jury: BoundedVec<AccountId, Self::MaxJurySize>,
        _specific_ids: BoundedVec<Self::SpecificId, Self::MaxSpecifics>,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
}

pub struct MockJurySelector;
impl pallet_fellowship::traits::SelectJury<AccountId> for MockJurySelector {
    type JurySize = MaxJuryMembers;
    fn select_jury() -> BoundedVec<AccountId, Self::JurySize> {
        BoundedVec::new()
    }
}
