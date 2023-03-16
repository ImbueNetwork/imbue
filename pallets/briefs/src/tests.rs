use crate::mock::*;
use crate::*;
use common_types::CurrencyId;
use frame_support::pallet_prelude::Hooks;
use frame_support::{assert_noop, assert_ok, once_cell::sync::Lazy};
use sp_core::H256;

pub fn gen_hash(seed: u8) -> H256 {
    H256::from([seed; 32])
}

static DEPOSIT_OK: Lazy<Balance> = Lazy::new(|| <Test as Config>::MinimumDeposit::get() + 1000);
static BOUNTY_OK: Lazy<Balance> = Lazy::new(|| <Test as Config>::MinimumBounty::get() + 1000);

#[test]
fn approve_freelancer_not_root() {
    build_test_externality().execute_with(|| {
        assert_noop!(BriefsMod::approve_account(RuntimeOrigin::signed(*ALICE), *BOB), Error::<Test>::BadOrigin);
    });
}

#[test]
fn approve_freelancer_as_root() {
    build_test_externality().execute_with(|| {
        assert_ok!(BriefsMod::approve_account(RuntimeOrigin::root(), *BOB));
    });
}



fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        BriefsMod::on_initialize(System::block_number());
    }
}
