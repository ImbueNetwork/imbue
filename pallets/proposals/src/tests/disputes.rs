use frame_support::{assert_noop, assert_ok};
use crate::{mock::*, *};
use common_types::CurrencyId;
use test_utils::*;
use pallet_disputes::DisputeResult;

#[test]
fn raise_dispute_not_contributor() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        
        assert_noop!(Proposals::raise_dispute(RuntimeOrigin::signed(*JOHN), project_key, milestone_keys), Error::<Test>::OnlyContributorsCanRaiseDispute);
    })
}

#[test]
fn raise_dispute_project_doesnt_exist() {
    build_test_externality().execute_with(|| {
        assert_noop!(Proposals::raise_dispute(RuntimeOrigin::signed(*JOHN), 0, vec![0u32].try_into().unwrap()), Error::<Test>::ProjectDoesNotExist);
    })
}

#[test]
fn raise_dispute_milestone_already_in_dispute() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys));
        for i in milestones.iter() {
            assert_noop!(Proposals::raise_dispute(RuntimeOrigin::signed(*CHARLIE), project_key, vec![i].try_into().unwrap()), Error::<Test>::MilestonesAlreadyInDispute);
        }
    })
}

#[test]
fn raise_dispute_invalid_milestone_key() {
    build_test_externality().execute_with(|| {

    })
}

#[test]
fn raise_dispute_cant_raise_on_approved_milestone() {
    build_test_externality().execute_with(|| {

    })
}

#[test]
fn on_dispute_complete_success_removes_dispute_status() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);
        assert!(!ProjectsInDispute::<Test>::contains_key(project_key));
    })
}


#[test]
fn on_dispute_complete_failure_removes_dispute_status() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Failure);
        assert!(!ProjectsInDispute::<Test>::contains_key(project_key));
    })
}

#[test]
fn dispute_success_does_not_cancel_project() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();

        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);

        let project = Projects::<Test>::get(project_key).unwrap();
        assert!(!project.cancelled);
    })
}

#[test]
fn dispute_success_approves_milestone_for_refund_but_only_ones_specified() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (1u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();

        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);

    })
}

#[test]
fn dispute_success_returns_non_zero_weight() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();

        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let weight = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);
        assert!(weight != <Weight as Zero>::zero());
    })
}

#[test]
fn raise_dispute_prevents_milestone_voting() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let submitted_milestone_key = 0;
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            submitted_milestone_key
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            submitted_milestone_key,
            true
        ));
        let dispute_milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, dispute_milestone_keys.clone()));
        
        assert_noop!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*CHARLIE),
            project_key,
            submitted_milestone_key,
            true
        ), Error::<Test>::MilestonesAlreadyInDispute);
    })
}


#[test]
fn raise_dispute_allows_milestone_voting_on_non_disputed_milestones() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let submitted_milestone_keys = [0, 1];
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            submitted_milestone_keys[0]
        ));

        let dispute_milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (2u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, dispute_milestone_keys.clone()));
        
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            submitted_milestone_keys[1]
        ));

        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*CHARLIE),
            project_key,
            submitted_milestone_keys[0],
            true
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            submitted_milestone_keys[1],
            true
        ));
    })
}


#[test]
fn raise_dispute_allows_submission() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let milestone_key = 0;
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();

        let dispute_milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, dispute_milestone_keys.clone()));
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone_key
        ));
    })
}

#[test]
fn you_can_actually_refund_after_dispute_success() {
    build_test_externality().execute_with(|| {

    })
}
