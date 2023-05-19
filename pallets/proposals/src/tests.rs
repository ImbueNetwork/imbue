use crate::{mock::*, Error, Event, Config};
use frame_support::{assert_noop, assert_ok};
use common_types::CurrencyId;
use orml_traits::{MultiReservableCurrency, MultiCurrency};

#[test]
fn submit_milestone() {
    new_test_ext().execute_with(|| {


    });
}


fn create_project(
    currency_id: CurrencyId,
    contributions: ContributionsFor<T>,
    brief_hash: H256,
    benificiary: AccountIdOf<T>,
    proposed_milestones: Vec<ProposedMilestone>,
    funding_type: FundingType
) 
{
    assert_ok!(<crate::Pallet::<T> as IntoProposal>::convert_to_proposal(
        currency_id,
        contributions,
        brief_hash,
        benificiary,
        proposed_milestones,
        funding_type,
    ))
}