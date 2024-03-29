use crate as pallet_deposits;
use crate::traits::{DepositCalculator, DepositHandler};
use common_types::CurrencyId;
use frame_support::traits::{ConstU16, ConstU64, Nothing};
use frame_support::{pallet_prelude::*, parameter_types};
use orml_traits::MultiCurrency;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Deposits: pallet_deposits,
        Tokens: orml_tokens,
    }
);

pub type AccountId = u128;
pub type Balance = u64;

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

parameter_types! {
    pub DepositSlashAccount: AccountId = 66;

}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo, Copy)]
pub enum StorageItem {
    CrowdFund,
    Brief,
    Grant,
    Project,
    Unsupported,
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
    fn calculate_deposit(
        item: Self::StorageItem,
        currency: CurrencyId,
    ) -> Result<Balance, DispatchError> {
        if currency != CurrencyId::Native {
            return Err(crate::pallet::Error::<Test>::UnsupportedCurrencyType.into());
        }
        if item == StorageItem::Unsupported {
            return Err(crate::pallet::Error::<Test>::UnsupportedStorageType.into());
        }
        Ok(10_000u64)
    }
}

struct MockDepositHandler<T>(T);
impl<T: crate::Config> DepositHandler<crate::BalanceOf<T>, crate::AccountIdOf<T>>
    for MockDepositHandler<T>
{
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

pub static ALICE: AccountId = 125;
pub static BOB: AccountId = 126;
pub static CHARLIE: AccountId = 127;

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
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

impl<T: crate::Config> DepositHandler<crate::BalanceOf<T>, crate::AccountIdOf<T>> for T {
    type DepositId = u64;
    type StorageItem = StorageItem;
    fn take_deposit(
        _who: crate::AccountIdOf<T>,
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
