use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use pallet_disputes::DisputeResult;
use test_utils::*;

#[test]
fn initiator_dispute_complete_sets_milestones_to_approved() {
    build_test_externality().execute_with(|| {
        let per_contribution = 100000u128;
        let contributions = get_contributions::<Test>(vec![BOB, CHARLIE], per_contribution);
        let milestones = get_milestones(10);
        let jury = vec![JURY_1, JURY_2];
        let initiator = ALICE;

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (1u32
            ..milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();

        assert_ok!(Proposals::raise_dispute(
            RuntimeOrigin::signed(initiator),
            project_key,
            milestone_keys.clone()
        ));

        let _ = complete_dispute::<Test>(
            initiator,
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );

        let project = Projects::<Test>::get(project_key).unwrap();

        for milestone in project.milestones.iter().for_each(|(key, milestone)|{
            if milestone_keys.contains(&key) {
                assert!(milestone.is_approved, "dispute success for initiator should approve milestones.")
            } else {
                assert!(!milestone.is_approved, "other milestones should be left unapproved.")
            }
        })
    })
}


