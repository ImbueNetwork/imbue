
use crate::mock::*;
use crate::*;
use common_types::CurrencyId;
use frame_support::{assert_noop, assert_ok, once_cell::sync::Lazy};
use sp_core::H256;
use frame_support::pallet_prelude::Hooks;


pub fn gen_hash(seed: u8) -> H256 {
	H256::from([seed; 32])
}

static DEPOSIT_OK: Lazy<Balance> = Lazy::new(||{<Test as Config>::MinimumDeposit::get() + 1000});
static BOUNTY_OK: Lazy<Balance> = Lazy::new(||{<Test as Config>::MinimumBounty::get() + 1000});

#[test]
fn brief_submit_deposit_below_minimum() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let d_below_minimum = <Test as Config>::MinimumDeposit::get() - amount;
		assert_noop!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), 100_000, d_below_minimum, CurrencyId::Native), Error::<Test>::DepositBelowMinimum);
	});
}


#[test]
fn brief_submit_bounty_below_minumum() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let b_below_minimum = <Test as Config>::MinimumBounty::get() - amount;
		assert_noop!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), b_below_minimum, *DEPOSIT_OK, CurrencyId::Native), Error::<Test>::BountyBelowMinimum);
	});
}


#[test]
fn brief_submit_contribution_more_than_bounty() {
    build_test_externality().execute_with(|| {
		let amount: Balance = 10; 
		let b_above_minimum = <Test as Config>::MinimumBounty::get() + amount;
		let d_above_bounty = b_above_minimum + amount;
		assert_noop!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), b_above_minimum, d_above_bounty, CurrencyId::Native), Error::<Test>::ContributionMoreThanBounty);
	});
}

#[test]
fn brief_submit_already_exists_in_block() {
    build_test_externality().execute_with(|| {
		// Assert that we can submit a brief with correct parameters
		assert_ok!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native));
		assert_noop!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native), Error::<Test>::BriefAlreadyExists);
	});
}


#[test]
fn brief_submit_already_exists_future_blocks() {
    build_test_externality().execute_with(|| {

		// Assert that we can submit a brief with correct parameters
		assert_ok!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native));

		// Assert that when we are on future blocks the same brief cannot be set.
		run_to_block(System::block_number() + 1);
		assert_noop!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native), Error::<Test>::BriefAlreadyExists);
		
		run_to_block(System::block_number()  + 1);
		assert_noop!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native), Error::<Test>::BriefAlreadyExists);
	});
}

// Tests for brief auction.
#[test]
fn brief_submit_auction_duplicate() {
    build_test_externality().execute_with(|| {
		assert_ok!(BriefsMod::submit_brief_auction(RuntimeOrigin::signed(*ALICE), gen_hash(1u8)));
		assert_noop!(BriefsMod::submit_brief_auction(RuntimeOrigin::signed(*ALICE), gen_hash(1u8)), Error::<Test>::BriefAlreadyExists);
	});
}

#[test]
fn brief_submit_application_ok() {
    build_test_externality().execute_with(|| {
		assert_ok!(BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native));
		assert_ok!(BriefsMod::submit_brief_auction(RuntimeOrigin::signed(*ALICE), gen_hash(2u8)));

		let _ = BriefsMod::approve_account(RuntimeOrigin::root(), *BOB);
		assert_ok!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(1u8)));
		assert_ok!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(2u8)));
	});
}

#[test]
fn brief_submit_application_ok_with_multiple_applicants() {
    build_test_externality().execute_with(|| {
		let _ = BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native);
		let _ = BriefsMod::submit_brief_auction(RuntimeOrigin::signed(*ALICE), gen_hash(2u8));

		let _ = BriefsMod::approve_account(RuntimeOrigin::root(), *BOB);
		let _ = BriefsMod::approve_account(RuntimeOrigin::root(), *CHARLIE);

		assert_ok!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(1u8)));
		assert_ok!(BriefsMod::submit_application(RuntimeOrigin::signed(*CHARLIE), gen_hash(1u8)));
		assert_ok!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(2u8)));
		assert_ok!(BriefsMod::submit_application(RuntimeOrigin::signed(*CHARLIE), gen_hash(2u8)));
	});
}

#[test]
fn brief_submit_application_unapproved() {
    build_test_externality().execute_with(|| {
		let _ = BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native);
		let _ = BriefsMod::submit_brief_auction(RuntimeOrigin::signed(*ALICE), gen_hash(2u8));

		assert_noop!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(1u8)), Error::<Test>::OnlyApprovedAccountPermitted);
		assert_noop!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(2u8)), Error::<Test>::OnlyApprovedAccountPermitted);
	});
}

#[test]
fn brief_submit_application_brief_not_found() {
    build_test_externality().execute_with(|| {
		let _ = BriefsMod::approve_account(RuntimeOrigin::root(), *BOB);

		assert_noop!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(1u8)), Error::<Test>::BriefNotFound);
		assert_noop!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(2u8)), Error::<Test>::BriefNotFound);
	});
}

#[test]
fn brief_submit_application_already_applied() {
    build_test_externality().execute_with(|| {
		let _ = BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native);
		let _ = BriefsMod::submit_brief_auction(RuntimeOrigin::signed(*ALICE), gen_hash(2u8));

		let _ = BriefsMod::approve_account(RuntimeOrigin::root(), *BOB);

		assert_ok!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(1u8)));
		assert_noop!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(1u8)), Error::<Test>::AlreadyApplied);

		assert_ok!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(2u8)));
		assert_noop!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(2u8)), Error::<Test>::AlreadyApplied);
	});
}

#[test]
fn brief_submit_application_closed_for_applications() {
    build_test_externality().execute_with(|| {
		let _ = BriefsMod::submit_brief_direct(RuntimeOrigin::signed(*ALICE), gen_hash(1u8), *BOUNTY_OK, *DEPOSIT_OK, CurrencyId::Native);
		let _ = BriefsMod::submit_brief_auction(RuntimeOrigin::signed(*ALICE), gen_hash(2u8));
		let _ = BriefsMod::approve_account(RuntimeOrigin::root(), *BOB);

		run_to_block(System::block_number() + <Test as Config>::ApplicationSubmissionTime::get() as u64);
		assert_noop!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(2u8)), Error::<Test>::BriefClosedForApplications);
		assert_noop!(BriefsMod::submit_application(RuntimeOrigin::signed(*BOB), gen_hash(1u8)), Error::<Test>::BriefClosedForApplications);
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
