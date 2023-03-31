use crate::mock::*;
use crate::tests::{gen_hash, get_milestones};
use crate::*;
use common_types::CurrencyId;

use frame_support::{assert_ok, bounded_vec};
use pallet_proposals::{Projects, RoundType};
use sp_core::H256;

use std::convert::TryInto;

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

#[test]
fn assert_state_from_brief_conversion_is_same_as_proposals_flow() {
    build_test_externality().execute_with(|| {
        let brief_id = H256::from([12; 32]);
        let milestones = get_milestones(10);
        let project_key = 0;
        let contribution_value: Balance = 10000;

        // Now we create a proposal with the same parameters.
        // The state of each of the project should be the same and therefore will function the same.
        assert_ok!(Proposals::create_project(
            RuntimeOrigin::signed(*ALICE),
            brief_id,
            milestones
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .try_into()
                .expect("proposed milestones are too long"),
            contribution_value,
            CurrencyId::Native,
        ));

        let _ = Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![0],
            RoundType::ContributionRound,
        )
        .unwrap();

        let _ = Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key,
            contribution_value,
        );
        let _ = Proposals::approve(
            RuntimeOrigin::root(),
            Some(1),
            project_key,
            Some(
                milestones
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("qed"),
            ),
        )
        .unwrap();

        let _ = BriefsMod::create_brief(
            RuntimeOrigin::signed(*ALICE),
            tests::get_brief_owners(1),
            *BOB,
            contribution_value,
            contribution_value,
            brief_id.clone(),
            CurrencyId::Native,
            milestones.clone(),
        );

        let _ = BriefsMod::commence_work(RuntimeOrigin::signed(*BOB), brief_id);
    });
}
