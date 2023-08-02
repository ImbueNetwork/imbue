
use crate::Runtime;

#[test]
fn ensure_maximum_milestones_are_consistent_grants() {
    let max_milestone_grants = <Runtime as pallet_grants::Config>::MaxMilestonesPerGrant::get();
    let max_milestone_proposals = <Runtime as pallet_proposals::Config>::MaxMilestonesPerProject::get();
    assert_eq!(max_milestone_grants, max_milestone_proposals, "The grants max milestones and project max milestones must be equal.");
}

#[test]
fn ensure_maximum_milestones_are_consistent_briefs() {
    let max_milestone_briefs = <Runtime as pallet_briefs::Config>::MaxMilestonesPerBrief::get();
    let max_milestone_proposals = <Runtime as pallet_proposals::Config>::MaxMilestonesPerProject::get();
    assert_eq!(max_milestone_briefs, max_milestone_proposals, "The briefs max milestones and project max milestones must be equal.");
}

#[test]
fn ensure_max_contributors_equal_max_approvers() {
    let max_contributors_proposals = <Runtime as pallet_proposals::Config>::MaximumContributorsPerProject::get();
    let max_approvers = <Runtime as pallet_grants::Config>::MaxApprovers::get();

    assert!(max_contributors_proposals >= max_approvers, "The max approvers must be less than or equal the max contributors.");
}

// A brief owner is used as the contibutors to a project so the maximums must be equal.
#[test]
fn ensure_max_contributors_equal_max_brief_owners() {
    let max_contributors_proposals = <Runtime as pallet_proposals::Config>::MaximumContributorsPerProject::get();
    let max_brief_owners = <Runtime as pallet_briefs::Config>::MaxBriefOwners::get();

    assert!(max_contributors_proposals >= max_brief_owners, "Max brief owners must be less than or equal to the the max contributors");
}



