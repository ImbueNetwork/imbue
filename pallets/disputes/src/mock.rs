use crate as pallet_disputes;
use common_types::CurrencyId;
use frame_support::once_cell::sync::Lazy;
use frame_support::traits::{ConstU16, Nothing};
use frame_support::{pallet_prelude::*, parameter_types};
use frame_system::EnsureRoot;
use orml_traits::MultiCurrency;
use sp_core::sr25519::{Public, Signature};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};
use sp_std::convert::{TryFrom, TryInto};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;
pub type Balance = u64;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        PalletDisputes: pallet_disputes,
        Tokens: orml_tokens,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ();
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = orml_tokens::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub MaxReasonLength: u32 = 100;
    pub MaxJurySize: u32 = 3;
    pub MaxSpecifics: u32 = 10;
    pub VotingTimeLimit: BlockNumber = 10;
    pub MaxDisputesPerBlock: u32 = 1000;
}

impl pallet_disputes::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type DisputeKey = u32;
    type SpecificId = u32;
    type MaxReasonLength = MaxReasonLength;
    type MaxJurySize = MaxJurySize;
    type MaxSpecifics = MaxSpecifics;
    type MaxDisputesPerBlock = MaxDisputesPerBlock;
    type VotingTimeLimit = VotingTimeLimit;
    type ForceOrigin = EnsureRoot<AccountId>;
    type DisputeHooks = Test;
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

pub static ALICE: Lazy<Public> = Lazy::new(|| Public::from_raw([125u8; 32]));
pub static BOB: Lazy<Public> = Lazy::new(|| Public::from_raw([126u8; 32]));
pub static CHARLIE: Lazy<Public> = Lazy::new(|| Public::from_raw([127u8; 32]));
pub static FERDIE: Lazy<Public> = Lazy::new(|| Public::from_raw([128u8; 32]));
pub static EMPTY: Lazy<Public> = Lazy::new(|| Public::from_raw([66u8; 32]));

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        let initial_balance = 100_000_000_000_000u64;
        System::set_block_number(1);
        let _ = Tokens::deposit(CurrencyId::Native, &ALICE, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &BOB, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &CHARLIE, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &FERDIE, initial_balance);
    });
    ext
}

impl crate::traits::DisputeHooks<u32> for Test {
    fn on_dispute_complete(
        _dispute_key: u32,
        _dispute_result: crate::pallet::DisputeResult,
    ) -> Weight {
        <Weight as Default>::default()
    }
}

impl crate::WeightInfoT for () {
    fn vote_on_dispute() -> Weight {
        <Weight as Default>::default()
    }
    fn extend_dispute() -> Weight {
        <Weight as Default>::default()
    }
    fn raise_dispute() -> Weight {
        <Weight as Default>::default()
    }
    fn force_succeed_dispute() -> Weight {
        <Weight as Default>::default()
    }
    fn force_fail_dispute() -> Weight {
        <Weight as Default>::default()
    }
    fn calculate_winner() -> Weight {
        <Weight as Default>::default()
    }
}
