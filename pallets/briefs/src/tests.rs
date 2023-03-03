use crate as pallet_briefs;
use crate::mock::*;
use crate::*;
use common_types::CurrencyId;
use frame_support::{assert_noop, assert_ok};
use frame_system::{Origin};

#[test]
fn brief_submit_deposit_below_minimum() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let d_below_minimum = <Test as Config>::MinimumDeposit::get() - amount;
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), 0, 100_000, d_below_minimum, CurrencyId::Native), Error::<Test>::DepositBelowMinimum);
	});
}


#[test]
fn brief_submit_bounty_below_minumum() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let b_below_minimum = <Test as Config>::MinimumBounty::get() - amount;
		let d_above_minimum = <Test as Config>::MinimumDeposit::get() + amount;
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), 0, b_below_minimum, d_above_minimum, CurrencyId::Native), Error::<Test>::DepositBelowMinimum);
	});
}


#[test]
fn brief_submit_contribution_more_than_bounty() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let b_above_minimum = <Test as Config>::MinimumBounty::get() + amount;
		let d_above_bounty = b_above_minimum + amount;
		assert_noop!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), 0, b_above_minimum, d_above_bounty, CurrencyId::Native), Error::<Test>::DepositBelowMinimum);
	});
}

#[test]
fn brief_submit_already_exists() {
    build_test_externality().execute_with(|| {
		let d_above_minimum = <Test as Config>::MinimumDeposit::get() + amount;
		let b_above_minimum = <Test as Config>::MinimumBounty::get() + amount;

		assert_ok!(BriefsMod::submit_brief(RuntimeOrigin::signed(*ALICE), 0, b_above_minimum, d_above_minimum, CurrencyId::Native));
		assert!(true)
	});
}


#[test]
fn brief_submit_not_enough_funds() {
    build_test_externality().execute_with(|| {
		assert!(true)
	});
}
