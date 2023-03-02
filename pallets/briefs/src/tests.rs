
use crate::{mock::*};
use crate::*;
use common_types::CurrencyId;

pub fn submit_brief(origin: OriginFor<T>, off_chain_ref_id: u32, bounty_total: BalanceOf<T>, initial_contribution: BalanceOf<T>, currency_id: CurrencyId) -> DispatchResult {

#[test]
fn brief_submit_deposit_below_minimum() {
    build_test_externality().execute_with(|| {
		let below_minimum = <T as Test>::MinimumDeposit::get() - 1u32;

		Briefs::submit_brief(Origin::signed(*ALICE), 0, 100_000, below_minimum, CurrencyId::Native);
	});
}


#[test]
fn brief_submit_bounty_below_minumum() {
    build_test_externality().execute_with(|| {
		assert!(true)
	});
}


#[test]
fn brief_submit_contribution_more_than_bounty() {
    build_test_externality().execute_with(|| {
		assert!(true)
	});
}

#[test]
fn brief_submit_already_exists() {
    build_test_externality().execute_with(|| {
		assert!(true)
	});
}


#[test]
fn brief_submit_not_enough_funds() {
    build_test_externality().execute_with(|| {
		assert!(true)
	});
}
