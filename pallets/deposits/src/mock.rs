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
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_core::sr25519::{Public, Signature};
use frame_support::once_cell::sync::Lazy;
use orml_traits::MultiCurrency;

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

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u64;
pub type BlockNumber = u64;

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

parameter_types!{
	pub DepositSlashAccount: AccountId = Public::from_raw([66u8; 32]);

}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo, Copy)]
	pub enum StorageItem {
	CrowdFund,
	Brief,
	Grant,
	Project,
}
pub(crate) type DepositId = u64;

impl pallet_deposits::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;
	type StorageItem = StorageItem;
	type DepositId = DepositId;
	type DepositCalculator = MockDepositCalculator;
	type DepositSlashAccount = DepositSlashAccount;
}

pub struct MockDepositCalculator;
impl DepositCalculator<Balance> for MockDepositCalculator {
	type StorageItem = StorageItem;
    fn calculate_deposit(_item: Self::StorageItem, _currency: CurrencyId) -> Balance {
		// TODO:
		10_000u64
	}
}

struct MockDepositHandler<T>(T);
impl<T: crate::Config> DepositHandler<crate::BalanceOf<T>, crate::AccountIdOf<T>> for MockDepositHandler<T> {
    type DepositId = T::DepositId;
    type StorageItem = T::StorageItem;
    fn take_deposit(
        _who: crate::AccountIdOf<T>,
        _storage_item: Self::StorageItem,
        _currency_id: CurrencyId,
    ) -> Result<T::DepositId, DispatchError> {
        todo!()
    }
    fn return_deposit(_deposit_id: Self::DepositId) -> DispatchResult {
        todo!()
    }
    fn slash_reserve_deposit(_deposit_id: Self::DepositId) -> DispatchResult {
        todo!()
    }
}

pub static ALICE: Lazy<Public> = Lazy::new(|| Public::from_raw([125u8; 32]));
pub static BOB: Lazy<Public> = Lazy::new(|| Public::from_raw([126u8; 32]));
pub static CHARLIE: Lazy<Public> = Lazy::new(|| Public::from_raw([127u8; 32]));

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
