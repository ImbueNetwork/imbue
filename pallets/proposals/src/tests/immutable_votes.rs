use crate::{mock::*, *};
use common_types::CurrencyId;
use frame_support::{assert_noop, assert_ok};
use test_utils::*;

#[test]
fn individual_votes_new_inserts_all_milestone_keys() {
    build_test_externality().execute_with(|| {
        let milestone_keys: BoundedVec<MilestoneKey, <Test as Config>::MaxMilestonesPerProject> =
            vec![0, 1, 2, 3]
                .try_into()
                .expect("should be smaller than bound.");
        let i = ImmutableIndividualVotes::<Test>::new(milestone_keys.clone()).unwrap();
        for key in milestone_keys {
            assert!(
                i.as_ref().get(&key).unwrap().is_empty(),
                "A milestone key should have been inserted, and the map must be empty."
            );
        }
    })
}

#[test]
fn individual_votes_insert_vote_works() {
    build_test_externality().execute_with(|| {
        let milestone_keys: BoundedVec<MilestoneKey, <Test as Config>::MaxMilestonesPerProject> =
            vec![0, 1, 2, 3]
                .try_into()
                .expect("should be smaller than bound.");
        let voting_key = milestone_keys[0];
        let mut i = ImmutableIndividualVotes::<Test>::new(milestone_keys.clone()).unwrap();
        assert_ok!(i.insert_individual_vote(voting_key, &ALICE, true));
        assert_ok!(i.insert_individual_vote(voting_key, &BOB, false));
        assert_eq!(
            i.as_ref().get(&voting_key).unwrap().get(&ALICE).unwrap(),
            &true,
            "ALICE vote should exist and been voted true,"
        );
        assert_eq!(
            i.as_ref().get(&voting_key).unwrap().get(&BOB).unwrap(),
            &false,
            "BOB vote should exist and been voted false."
        );
    })
}

#[test]
fn individual_votes_votes_are_immutable() {
    build_test_externality().execute_with(|| {
        let milestone_keys: BoundedVec<MilestoneKey, <Test as Config>::MaxMilestonesPerProject> =
            vec![0, 1, 2, 3]
                .try_into()
                .expect("should be smaller than bound.");
        let voting_key = milestone_keys[0];
        let mut i = ImmutableIndividualVotes::<Test>::new(milestone_keys.clone()).unwrap();
        i.insert_individual_vote(voting_key, &ALICE, true).unwrap();
        assert_noop!(
            i.insert_individual_vote(voting_key, &ALICE, false),
            Error::<Test>::VotesAreImmutable
        );
    })
}

#[test]
fn individual_votes_cannot_vote_on_non_existant_milestone() {
    build_test_externality().execute_with(|| {
        let milestone_keys: BoundedVec<MilestoneKey, <Test as Config>::MaxMilestonesPerProject> =
            vec![0, 1, 2, 3]
                .try_into()
                .expect("should be smaller than bound.");
        let voting_key = 4;
        let mut i = ImmutableIndividualVotes::<Test>::new(milestone_keys.clone()).unwrap();
        assert_noop!(
            i.insert_individual_vote(voting_key, &ALICE, true),
            Error::<Test>::IndividualVoteNotFound
        );
    })
}

#[test]
fn individual_votes_clear_votes_actually_clears() {
    build_test_externality().execute_with(|| {
        let milestone_keys: BoundedVec<MilestoneKey, <Test as Config>::MaxMilestonesPerProject> =
            vec![0, 1, 2, 3]
                .try_into()
                .expect("should be smaller than bound.");
        let voting_key = milestone_keys[0];
        let mut i = ImmutableIndividualVotes::<Test>::new(milestone_keys.clone()).unwrap();
        i.insert_individual_vote(voting_key, &ALICE, true).unwrap();
        i.insert_individual_vote(voting_key, &BOB, true).unwrap();
        i.clear_milestone_votes(voting_key);
        assert!(
            i.as_ref().get(&voting_key).unwrap().is_empty(),
            "The btree should have been emptied after a cleared vote."
        );
    })
}
