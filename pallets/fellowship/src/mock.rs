use crate as pallet_fellowship;
use common_types::CurrencyId;
use frame_support::traits::{ConstU16, Nothing};
use frame_system::EnsureRoot;
use orml_traits::MultiCurrency;
use sp_core::H256;
use sp_runtime::{
    BuildStorage,
    traits::{parameter_types, BlakeTwo256, IdentityLookup },
};
use sp_std::convert::{TryFrom, TryInto};

type Block = frame_system::mocking::MockBlock<Test>;
pub type BlockNumber = u64;
pub type Balance = u64;
pub type AccountId = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Fellowship: pallet_fellowship,
        Tokens: orml_tokens,
    }
);

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
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = orml_tokens::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub MaxCandidatesPerShortlist: u32 = 100;
    pub ShortlistPeriod: BlockNumber = 100;
    pub MembershipDeposit: Balance = 50_000_000;
    pub SlashAccount: AccountId = 1;
    pub BlockHashCount: BlockNumber = 250;
    pub DepositCurrencyId: CurrencyId = CurrencyId::Native;
}

impl pallet_fellowship::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MultiCurrency = Tokens;
    type ForceAuthority = EnsureRoot<AccountId>;
    type MaxCandidatesPerShortlist = MaxCandidatesPerShortlist;
    type ShortlistPeriod = ShortlistPeriod;
    type MembershipDeposit = MembershipDeposit;
    type DepositCurrencyId = DepositCurrencyId;
    type SlashAccount = SlashAccount;
    type WeightInfo = ();
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

pub static ALICE: AccountId = 125;
pub static BOB: AccountId = 126;
pub static CHARLIE: AccountId = 127;
pub static EMPTY: AccountId = 66;
pub static TREASURY: AccountId = 1;

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        let initial_balance = 100_000_000_000_000u64;
        System::set_block_number(1);
        let _ = Tokens::deposit(CurrencyId::Native, &ALICE, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &BOB, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &CHARLIE, initial_balance);
        let _ = Tokens::deposit(CurrencyId::Native, &TREASURY, initial_balance);
    });
    ext
}

use frame_support::pallet_prelude::Weight;
impl crate::traits::WeightInfoT for () {
    fn add_to_fellowship() -> Weight {
        <Weight as Default>::default()
    }
    fn force_add_fellowship() -> Weight {
        <Weight as Default>::default()
    }
    fn leave_fellowship() -> Weight {
        <Weight as Default>::default()
    }
    fn force_remove_and_slash_fellowship() -> Weight {
        <Weight as Default>::default()
    }
    fn add_candidate_to_shortlist() -> Weight {
        <Weight as Default>::default()
    }
    fn remove_candidate_from_shortlist() -> Weight {
        <Weight as Default>::default()
    }
    fn pay_deposit_to_remove_pending_status() -> Weight {
        <Weight as Default>::default()
    }
}
