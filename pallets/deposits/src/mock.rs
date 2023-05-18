use crate as pallet_deposits;
use frame_support::traits::{ConstU16, ConstU64, Nothing};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use crate::traits::DepositCalculator;
use common_types::CurrencyId;
use frame_support::{pallet_prelude::*, parameter_types};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Deposits: pallet_deposits,
		Tokens: orml_tokens,
	}
);

type AccountId = u64;
type Balance = u64;
type BlockNumber = u64;

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
	type AccountData = ();
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
    type CurrencyId = CurrencyId;
    type CurrencyHooks = ();
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type DustRemovalWhitelist = Nothing;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}


impl pallet_deposits::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;
	type DepositId = DepositId;
	type StorageItem = StorageItem;
	type DepositCalculator = MockDepositCalculator;
	type CurrencyId = CurrencyId;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub enum StorageItem {
	Project,
	Grant,
	Brief,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo, Encode, Decode)]
pub enum DepositId {
	Project(u32),
	Grant(u32),
	Brief(u32),
}

pub struct MockDepositCalculator;

impl DepositCalculator<Balance> for MockDepositCalculator {
	type CurrencyId = CurrencyId;
	type StorageItem = StorageItem;
    fn calculate_deposit(item: Self::StorageItem, currency: Self::CurrencyId) -> Balance {
		10_000u64
	}
}


// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
