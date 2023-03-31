use crate::mock::*;
use crate::tests::get_milestones;
use crate::*;
use common_types::CurrencyId;

use frame_support::assert_ok;
use pallet_proposals::Projects;

// all the integration tests for a brief to proposal conversion
#[test]
fn create_proposal_from_brief() {
    build_test_externality().execute_with(|| {
        let brief_id = gen_hash(100);
        let contribution_value: Balance = 10000;

        let _ = BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            tests::get_brief_owners(1),
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

        let created_project = Projects::<Test>::get(1).unwrap();

        assert_eq!(created_project.agreement_hash, brief_id);
        assert_eq!(created_project.approved_for_funding, true);
        assert_eq!(created_project.required_funds, contribution_value);
    });
}
