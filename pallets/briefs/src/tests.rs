use crate::*;
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn submit_basic_brief() {
    build_test_externality().execute_with(|| {
		//Briefs::submit_brief()
	});
}
origin: OriginFor<T>, brief_id: BriefId, bounty_total: BalanceOf<T>, initial_contribution: BalanceOf<T>, currency_id: CurrencyId) -> DispatchResult {
