
use crate::mock::*;
use crate::*;
use common_types::CurrencyId;
use frame_support::{assert_noop, assert_ok, once_cell::sync::Lazy};
use sp_core::H256;
use frame_support::pallet_prelude::Hooks;

static TESTHASH: Lazy<H256> = Lazy::new(||{H256::from([1; 32])});

#[test]
fn brief_submit_deposit_below_minimum() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let d_below_minimum = <Test as Config>::MinimumDeposit::get() - amount;
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), *TESTHASH, 100_000, d_below_minimum, CurrencyId::Native), Error::<Test>::DepositBelowMinimum);
	});
}


#[test]
fn brief_submit_bounty_below_minumum() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let b_below_minimum = <Test as Config>::MinimumBounty::get() - amount;
		let d_above_minimum = <Test as Config>::MinimumDeposit::get() + amount;
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), *TESTHASH, b_below_minimum, d_above_minimum, CurrencyId::Native), Error::<Test>::BountyBelowMinimum);
	});
}


#[test]
fn brief_submit_contribution_more_than_bounty() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let b_above_minimum = <Test as Config>::MinimumBounty::get() + amount;
		let d_above_bounty = b_above_minimum + amount;
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), *TESTHASH, b_above_minimum, d_above_bounty, CurrencyId::Native), Error::<Test>::ContributionMoreThanBounty);
	});
}

#[test]
fn brief_submit_already_exists_in_block() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let d_above_minimum = <Test as Config>::MinimumDeposit::get() + amount;
		let b_above_minimum = <Test as Config>::MinimumBounty::get() + amount;

		// Assert that we can submit a brief with correct parameters
		assert_ok!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), *TESTHASH, b_above_minimum, d_above_minimum, CurrencyId::Native));
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), *TESTHASH, b_above_minimum, d_above_minimum, CurrencyId::Native), Error::<Test>::BriefAlreadyExists);
	});
}


#[test]
fn brief_submit_already_exists_future_blocks() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let d_above_minimum = <Test as Config>::MinimumDeposit::get() + amount;
		let b_above_minimum = <Test as Config>::MinimumBounty::get() + amount;

		// Assert that we can submit a brief with correct parameters
		assert_ok!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), *TESTHASH, b_above_minimum, d_above_minimum, CurrencyId::Native));

		// Assert that when we are on future blocks the same brief cannot be set.
		run_to_block(System::block_number() + 1);
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), *TESTHASH, b_above_minimum, d_above_minimum, CurrencyId::Native), Error::<Test>::BriefAlreadyExists);
		
		run_to_block(System::block_number()  + 1);
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), *TESTHASH, b_above_minimum, d_above_minimum, CurrencyId::Native), Error::<Test>::BriefAlreadyExists);
	});
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Proposals::on_initialize(System::block_number());
		BriefsMod::on_initialize(System::block_number());
    }
}
