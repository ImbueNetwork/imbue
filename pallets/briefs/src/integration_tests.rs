use crate::mock::*;
use crate::test_utils::gen_hash;

use crate::tests::{get_brief_owners, get_milestones};
use common_types::CurrencyId;
use frame_support::{assert_ok};
use pallet_proposals::{Projects};

// all the integration tests for a brief to proposal conversion
#[test]
fn create_proposal_from_brief() {
    build_test_externality().execute_with(|| {
        let brief_id = gen_hash(100);
        let contribution_value: Balance = 10000;

        let _ = BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            get_brief_owners(1),
            *ALICE,
            contribution_value,
            contribution_value,
            brief_id.clone(),
            CurrencyId::Native,
            get_milestones(10),
        );

        assert_ok!(BriefsMod::commence_work(
            RuntimeOrigin::signed(*ALICE),
            brief_id
        ));

        // TODO: integration tests
        assert!(Projects::<Test>::get(1).is_some());
        
    });
}
