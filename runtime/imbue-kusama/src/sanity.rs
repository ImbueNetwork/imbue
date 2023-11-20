use crate::Runtime;
use common_runtime::MAXIMUM_BLOCK_WEIGHT;
use pallet_proposals::{WeightInfo as PWeightInfo, WeightInfoT};
use sp_arithmetic::Percent;

#[test]
fn ensure_maximum_milestones_are_consistent_grants() {
    let max_milestone_grants = <Runtime as pallet_grants::Config>::MaxMilestonesPerGrant::get();
    let max_milestone_proposals =
        <Runtime as pallet_proposals::Config>::MaxMilestonesPerProject::get();
    assert_eq!(
        max_milestone_grants, max_milestone_proposals,
        "The grants max milestones and project max milestones must be equal."
    );
}

#[test]
fn ensure_maximum_milestones_are_consistent_briefs() {
    let max_milestone_briefs = <Runtime as pallet_briefs::Config>::MaxMilestonesPerBrief::get();
    let max_milestone_proposals =
        <Runtime as pallet_proposals::Config>::MaxMilestonesPerProject::get();
    assert_eq!(
        max_milestone_briefs, max_milestone_proposals,
        "The briefs max milestones and project max milestones must be equal."
    );
}

#[test]
fn ensure_max_contributors_equal_max_approvers() {
    let max_contributors_proposals =
        <Runtime as pallet_proposals::Config>::MaximumContributorsPerProject::get();
    let max_approvers = <Runtime as pallet_grants::Config>::MaxApprovers::get();

    assert!(
        max_contributors_proposals >= max_approvers,
        "The max approvers must be less than or equal the max contributors."
    );
}

// A brief owner is used as the contibutors to a project so the maximums must be equal.
#[test]
fn ensure_max_contributors_equal_max_brief_owners() {
    let max_contributors_proposals =
        <Runtime as pallet_proposals::Config>::MaximumContributorsPerProject::get();
    let max_brief_owners = <Runtime as pallet_briefs::Config>::MaxBriefOwners::get();

    assert!(
        max_contributors_proposals >= max_brief_owners,
        "Max brief owners must be less than or equal to the the max contributors"
    );
}

#[test]
fn ensure_proposals_initialize_is_less_than_10_percent_block() {
    let multiplier = <Runtime as pallet_proposals::Config>::ExpiringProjectRoundsPerBlock::get();
    let ref_time =
        <PWeightInfo<Runtime> as WeightInfoT>::on_initialize().ref_time() * multiplier as u64;
    let proof_size =
        <PWeightInfo<Runtime> as WeightInfoT>::on_initialize().proof_size() * multiplier as u64;

    let max_ref_time = Percent::from_percent(10u8).mul_floor(MAXIMUM_BLOCK_WEIGHT.ref_time());
    let max_proof_size = Percent::from_percent(10u8).mul_floor(MAXIMUM_BLOCK_WEIGHT.proof_size());

    assert!(
        ref_time <= max_ref_time,
        "ExpiringProjectRoundsPerBlock is exceeding ref time limits."
    );
    assert!(
        proof_size <= max_proof_size,
        "ExpiringProjectRoundsPerBlock is exceeding proof size limits."
    );
}

#[test]
fn test_select_jury_is_random() {
    new_test_ext().execute_with(|| {
        let size: u8 = 255;
        for i in 0..size {
            assert_ok!(Fellowship::force_add_fellowship(
                RuntimeOrigin::root(),
                Public::from_raw([i as u8; 32]),
                Role::Freelancer,
                10
            ));
        }
        let jury_1 = <PseudoRandomJurySelector<Runtime> as SelectJury<AccountId>>::select_jury();
        let jury_2 = <PseudoRandomJurySelector<Runtime> as SelectJury<AccountId>>::select_jury();

        assert!(jury_1[0] != jury_2[0]);

        let jury_1 = <PseudoRandomJurySelector<Runtime> as SelectJury<AccountId>>::select_jury();
        let jury_2 = <PseudoRandomJurySelector<Runtime> as SelectJury<AccountId>>::select_jury();

        assert!(jury_1 != jury_2);
    });
}

#[test]
fn test_select_jury_stress_test() {
    new_test_ext().execute_with(|| {
        let size: u8 = 255;
        for i in 0..size {
            assert_ok!(Fellowship::force_add_fellowship(
                RuntimeOrigin::root(),
                Public::from_raw([i as u8; 32]),
                Role::Freelancer,
                10
            ));
        }
        for _ in 0..u8::MAX {
            <PseudoRandomJurySelector<Runtime> as SelectJury<AccountId>>::select_jury(255);
        }
    });
}

#[test]
fn test_select_jury_select_correct_amount_in_normal_conditions() {
    new_test_ext().execute_with(|| {
        for i in 0..255 {
            assert_ok!(Fellowship::force_add_fellowship(
                RuntimeOrigin::root(),
                Public::from_raw([i as u8; 32]),
                Role::Freelancer,
                10
            ));
        }

        let jury = <PseudoRandomJurySelector<Runtime> as SelectJury<AccountId>>::select_jury(50);
        assert_eq!(jury.len(), <>, "jury members should be exactly 50.")
    });
}

#[test]
fn test_select_jury_doesnt_panic_amount_more_than_length() {
    new_test_ext().execute_with(|| {
        let size: u8 = 100;
        for i in 0..size {
            assert_ok!(Fellowship::force_add_fellowship(
                RuntimeOrigin::root(),
                Public::from_raw([i as u8; 32]),
                Role::Freelancer,
                10
            ));
        }
        let jury = <PseudoRandomJurySelector<Runtime> as SelectJury<AccountId>>::select_jury();
        assert_eq!(jury.len(), 100, "jury size should have been truncated to size of roles or not enough roles have been selected.");
    });
}

