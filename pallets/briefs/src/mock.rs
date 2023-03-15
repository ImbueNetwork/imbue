
use crate as pallet_briefs;
use frame_support::{
    parameter_types,
    traits::{ConstU32, Nothing},
    weights::{ConstantMultiplier, IdentityFee},
    PalletId,
};

use frame_system::EnsureRoot;
use sp_core::{sr25519::Signature, H256};

use sp_std::{
    convert::{TryFrom, TryInto},
    str,
    vec::Vec,
};
use frame_support::once_cell::sync::Lazy;
use common_types::CurrencyId;
use sp_core::sr25519;
use sp_runtime::{
    testing::{Header},
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify, AccountIdConversion},
    BuildStorage
};
use crate::mock::sp_api_hidden_includes_construct_runtime::hidden_include::traits::GenesisBuild;
use crate::traits::{BriefEvolver};
use crate::pallet::{IpfsHash};
use sp_runtime::DispatchResult;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type BlockNumber = <Test as frame_system::Config>::BlockNumber;
pub type Amount = i128;
pub type Balance = u64;

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
    type  BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type RuntimeEvent = RuntimeEvent;
    type  OnNewAccount = ();
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
    type Moment = u64;
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
}

impl pallet_briefs::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RMultiCurrency = Tokens;
    type MinimumDeposit = MinimumDeposit;
    type MinimumBounty = MinimumBounty;
    type BriefHasher = BlakeTwo256;
    type AuthorityOrigin = EnsureRoot<AccountId>;
    type BriefEvolver = DummyBriefEvolver;
    type MaxBriefOwners = MaxBriefOwners;
}

pub struct DummyBriefEvolver;

impl BriefEvolver<AccountId, Balance, BlockNumber> for DummyBriefEvolver {
    fn convert_to_proposal(brief_owner: AccountId, bounty: Balance, created_at: BlockNumber, ipfs_hash: IpfsHash) -> Result<(), ()> {
        // Perform the necessary logic here
        Ok(())
    }
}

parameter_types! {
    pub const UnitWeightCost: u64 = 10;
    pub const MaxInstructions: u32 = 100;
}
pub static ALICE : Lazy<sr25519::Public> = Lazy::new(||{sr25519::Public::from_raw([1u8; 32])});
pub static BOB : Lazy<sr25519::Public> = Lazy::new(||{sr25519::Public::from_raw([2u8; 32])});
pub static CHARLIE : Lazy<sr25519::Public> = Lazy::new(||{sr25519::Public::from_raw([10u8; 32])});

pub(crate) fn build_test_externality() -> sp_io::TestExternalities {

	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	
    GenesisConfig::default()
		.assimilate_storage(&mut t)
		.unwrap();
        
    orml_tokens::GenesisConfig::<Test> {
        balances: {
            vec![*ALICE, *BOB, *CHARLIE].into_iter().map(|id| {
            (id, CurrencyId::Native, 100000)
        }).collect::<Vec<_>>()},
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
