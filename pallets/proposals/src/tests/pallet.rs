use frame_support::{assert_noop, assert_ok};

use crate::{mock::*, *};
use common_types::CurrencyId;
use test_utils::*;

pub fn run_to_block(n: BlockNumber) {
    while System::block_number() < n {
        IdentityPallet::on_finalize(System::block_number());
        Proposals::on_finalize(System::block_number());
        TransactionPayment::on_finalize(System::block_number());
        Currencies::on_finalize(System::block_number());
        Tokens::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        Tokens::on_initialize(System::block_number());
        Currencies::on_initialize(System::block_number());
        TransactionPayment::on_initialize(System::block_number());
        Proposals::on_initialize(System::block_number());
        IdentityPallet::on_initialize(System::block_number());
    }
}

#[test]
fn submit_milestone_milestone_doesnt_exist() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_noop!(
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, 11),
            Error::<Test>::MilestoneDoesNotExist
        );
    });
}

#[test]
fn submit_milestone_no_project() {
    build_test_externality().execute_with(|| {
        assert_noop!(
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), 0, 1),
            Error::<Test>::ProjectDoesNotExist
        );
    });
}

#[test]
fn submit_milestone_not_initiator() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_noop!(
            Proposals::submit_milestone(RuntimeOrigin::signed(BOB), project_key, 1),
            Error::<Test>::UserIsNotInitiator
        );
        assert_noop!(
            Proposals::submit_milestone(RuntimeOrigin::signed(DAVE), project_key, 1),
            Error::<Test>::UserIsNotInitiator
        );
    });
}

#[test]
fn submit_milestones_too_many_this_block() {
    build_test_externality().execute_with(|| {
        let max = <Test as Config>::ExpiringProjectRoundsPerBlock::get();
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);

        (0..=max).for_each(|i| {
            let project_key = create_project::<Test>(
                ALICE,
                cont.clone(),
                prop_milestones.clone(),
                CurrencyId::Native,
            );
            if i != max {
                assert_ok!(Proposals::submit_milestone(
                    RuntimeOrigin::signed(ALICE),
                    project_key,
                    1
                ));
            } else {
                assert_noop!(
                    Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, 1),
                    Error::<Test>::Overflow
                );
            }
        })
    });
}

#[test]
fn submit_milestone_creates_non_bias_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            1
        ));
        let total_vote = MilestoneVotes::<Test>::get(project_key);
        let created_vote = total_vote.get(&1).expect("should exist");

        assert_eq!(created_vote.nay, 0, "initial vote should be default");
        assert_eq!(created_vote.yay, 0, "initial vote should be default");
    });
}

#[test]
fn submit_milestone_can_resubmit_during_voting_round() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true
        ));
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        let user_voted =
            UserHasVoted::<Test>::get((project_key, RoundType::VotingRound, milestone_key));
        dbg!(&user_voted);
        assert_eq!(
            user_voted.len(),
            0usize,
            "User votes should be defaulted on resubmission."
        );
        let total_vote = MilestoneVotes::<Test>::get(project_key);

        let group_vote = total_vote
            .get(&milestone_key)
            .expect("group vote should exist.");
        assert_eq!(
            group_vote,
            &<Vote<Balance> as Default>::default(),
            "Group vote should have defaulted on resubmission"
        );
    });
}

#[test]
fn submit_milestone_can_submit_again_after_failed_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            1
        ));
        let expiry_block = frame_system::Pallet::<Test>::block_number()
            + <Test as Config>::MilestoneVotingWindow::get();
        run_to_block(expiry_block + 1);
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            1
        ));
    });
}

#[test]
fn submit_milestone_cannot_submit_again_after_success_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(CHARLIE),
            project_key,
            milestone_key,
            true
        ));
        // The auto approval should have approved it here.
        let expiry_block = frame_system::Pallet::<Test>::block_number()
            + <Test as Config>::MilestoneVotingWindow::get();
        run_to_block(expiry_block + 1);
        assert_noop!(
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key),
            Error::<Test>::MilestoneAlreadyApproved
        );
    });
}

#[test]
fn ensure_milestone_vote_data_is_cleaned_after_autofinalisation_for() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true
        ));

        // Assert that the state is good before auto finalisation
        let exp_block = Rounds::<Test>::get((project_key, milestone_key), RoundType::VotingRound)
            .expect("There should be a round here for the project_key");
        assert!(RoundsExpiring::<Test>::get(exp_block).contains(&(
            project_key,
            RoundType::VotingRound,
            milestone_key
        )));

        let individual_votes = IndividualVoteStore::<Test>::get(project_key).unwrap();
        assert!(
            individual_votes
                .as_ref()
                .get(&milestone_key)
                .unwrap()
                .get(&BOB)
                .unwrap()
                == &true,
            "IndividualVoteStore has not been mutated correctly."
        );

        // Assert the storage has been cleared up after finalisation
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(CHARLIE),
            project_key,
            milestone_key,
            true
        ));

        assert!(
            Rounds::<Test>::get((project_key, milestone_key), RoundType::VotingRound).is_none()
        );
        assert_eq!(
            RoundsExpiring::<Test>::get(exp_block).len(),
            0,
            "This vec should have been emptied on auto finalisation."
        );
        let individual_votes = IndividualVoteStore::<Test>::get(project_key).unwrap();
        assert!(!individual_votes
            .as_ref()
            .get(&milestone_key)
            .unwrap()
            .contains_key(&BOB));
    });
}

#[test]
fn ensure_milestone_vote_data_is_cleaned_after_autofinalisation_against() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            false
        ));

        // Assert that the state is good before auto finalisation
        let exp_block = Rounds::<Test>::get((project_key, milestone_key), RoundType::VotingRound)
            .expect("There should be a round here for the project_key");
        assert!(RoundsExpiring::<Test>::get(exp_block).contains(&(
            project_key,
            RoundType::VotingRound,
            milestone_key
        )));

        let individual_votes = IndividualVoteStore::<Test>::get(project_key).unwrap();
        assert!(
            individual_votes
                .as_ref()
                .get(&milestone_key)
                .unwrap()
                .get(&BOB)
                .unwrap()
                == &false,
            "IndividualVoteStore has not been mutated correctly."
        );
        // Assert the storage has been cleared up after finalisation
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(CHARLIE),
            project_key,
            milestone_key,
            false
        ));

        assert!(
            Rounds::<Test>::get((project_key, milestone_key), RoundType::VotingRound).is_none()
        );
        assert_eq!(
            RoundsExpiring::<Test>::get(exp_block).len(),
            0,
            "This vec should have been emptied on auto finalisation."
        );
        let individual_votes = IndividualVoteStore::<Test>::get(project_key).unwrap();
        assert!(!individual_votes
            .as_ref()
            .get(&milestone_key)
            .unwrap()
            .contains_key(&BOB));
    });
}

#[test]
fn users_can_submit_multiple_milestones_and_vote_independantly() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key_0 = 0;
        let milestone_key_1 = 1;
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key_0
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key_0,
            true
        ));
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key_1
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key_1,
            true
        ));
        let total_votes = MilestoneVotes::<Test>::get(project_key);

        let vote_0 = total_votes
            .get(&milestone_key_0)
            .expect("vote 0 should exist");

        assert!(vote_0.yay == 100_000u64);
        assert!(vote_0.nay == 0u64);

        let vote_1 = total_votes
            .get(&milestone_key_1)
            .expect("vote 1 should exist");
        assert!(vote_1.yay == 100_000u64);
        assert!(vote_1.nay == 0u64);
    });
}

#[test]
fn vote_on_milestone_no_project() {
    build_test_externality().execute_with(|| {
        assert_noop!(
            Proposals::vote_on_milestone(RuntimeOrigin::signed(ALICE), 0, 0, true),
            Error::<Test>::ProjectDoesNotExist
        );
    });
}

#[test]
fn vote_on_milestone_before_round_starts_fails() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_noop!(
            Proposals::vote_on_milestone(
                RuntimeOrigin::signed(BOB),
                project_key,
                milestone_key,
                true
            ),
            Error::<Test>::VotingRoundNotStarted
        );
    });
}

#[test]
fn vote_on_milestone_after_round_end_fails() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let expiring_block = frame_system::Pallet::<Test>::block_number()
            + <Test as Config>::MilestoneVotingWindow::get();
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        run_to_block(expiring_block);
        assert_noop!(
            Proposals::vote_on_milestone(
                RuntimeOrigin::signed(BOB),
                project_key,
                milestone_key,
                true
            ),
            Error::<Test>::VotingRoundNotStarted
        );
    });
}

#[test]
fn vote_on_milestone_where_voting_round_is_active_but_not_the_correct_milestone() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            0
        ));
        assert_noop!(
            Proposals::vote_on_milestone(RuntimeOrigin::signed(BOB), project_key, 1, true),
            Error::<Test>::VotingRoundNotStarted
        );
    });
}

#[test]
fn if_double_submission_and_one_finalises_voting_on_the_second_can_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let expiring_block = frame_system::Pallet::<Test>::block_number()
            + <Test as Config>::MilestoneVotingWindow::get();
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            0
        ));
        run_to_block(frame_system::Pallet::<Test>::block_number() + 10);
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            1
        ));
        run_to_block(expiring_block);
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            1,
            true
        ));
    });
}

#[test]
fn vote_on_milestone_not_contributor() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        assert_noop!(
            Proposals::vote_on_milestone(
                RuntimeOrigin::signed(DAVE),
                project_key,
                milestone_key,
                true
            ),
            Error::<Test>::OnlyContributorsCanVote
        );
    });
}

#[test]
fn vote_on_milestone_actually_adds_to_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true
        ));
        let total_votes = MilestoneVotes::<Test>::get(project_key);
        let vote = total_votes.get(&milestone_key).expect("vote should exist");
        assert!(vote.yay == 100_000u64);
        assert!(vote.nay == 0u64);
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(CHARLIE),
            project_key,
            milestone_key,
            false
        ));
        let total_votes = MilestoneVotes::<Test>::get(project_key);
        let vote = total_votes.get(&milestone_key).expect("vote should exist");

        assert!(vote.yay == 100_000u64);
        assert!(vote.nay == 100_000u64);
    });
}

#[test]
fn withdraw_not_initiator() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(ALICE),
            project_key,
            milestone_key
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true
        ));
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(CHARLIE),
            project_key,
            milestone_key,
            true
        ));

        assert_noop!(
            Proposals::withdraw(RuntimeOrigin::signed(BOB), project_key),
            Error::<Test>::UserIsNotInitiator
        );
        assert_noop!(
            Proposals::withdraw(RuntimeOrigin::signed(DAVE), project_key),
            Error::<Test>::UserIsNotInitiator
        );
    });
}

#[test]
fn withdraw_only_transfers_approved_milestones() {
    build_test_externality().execute_with(|| {
        let per_contribution = 100_000;
        let cont = get_contributions::<Test>(vec![BOB, CHARLIE], per_contribution);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();
        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();
        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(CHARLIE),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();

        let alice_before =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));
        //validating the withdrawn flag set to true once the fund for the milestone is being withdrawn
        assert_eq!(Projects::<Test>::get(project_key).unwrap().milestones.get(&milestone_key).unwrap().withdrawn,true);

        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(per_contribution * 2 / 10);
        // total_contribution / number of milestones - fee
        let alice_expected_balance =
            alice_before + ((per_contribution * 2 / 10) as u64) - expected_fee as u64;
        assert_eq!(
            alice_after, alice_expected_balance,
            "Alice account is not the expected balance"
        );

        let project_account = crate::Pallet::<Test>::project_account_id(project_key);

        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &project_account),
            180_000,
            "funds havent been taken out of project as expected."
        );
    });
}

#[test]
fn withdraw_removes_project_after_all_funds_taken() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = get_milestones(1);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();
        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();
        assert!(Projects::<Test>::get(project_key).is_some());
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));

        assert!(
            Projects::<Test>::get(project_key).is_none(),
            "Project should have been removed after funds withdrawn."
        )
    });
}

#[test]
fn store_project_info_after_project_is_completed() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = get_milestones(1);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();
        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();
        assert!(Projects::<Test>::get(project_key).is_some());
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));

        if let Some((_account, projects)) = CompletedProjects::<Test>::iter().next() {
            assert_eq!(projects.len(), 1);
            assert!(projects.contains(&project_key));
        }
    });
}

#[test]
fn store_too_many_projects_for_account() {
    build_test_externality().execute_with(|| {
        let max = <Test as Config>::MaxProjectsPerAccount::get();
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = get_milestones(1);
        let milestone_key = 0;
        (0..=max).for_each(|i| {
            let project_key = create_project::<Test>(
                ALICE,
                cont.clone(),
                prop_milestones.clone(),
                CurrencyId::Native,
            );
            let _ = Proposals::submit_milestone(
                RuntimeOrigin::signed(ALICE),
                project_key,
                milestone_key,
            )
            .unwrap();
            let _ = Proposals::vote_on_milestone(
                RuntimeOrigin::signed(BOB),
                project_key,
                milestone_key,
                true,
            )
            .unwrap();

            if i != max {
                assert_ok!(Proposals::withdraw(
                    RuntimeOrigin::signed(ALICE),
                    project_key
                ));
            } else {
                assert_noop!(
                    Proposals::withdraw(RuntimeOrigin::signed(ALICE), project_key),
                    Error::<Test>::TooManyProjects
                );
            }
        })
    });
}

#[test]
fn withdraw_takes_imbue_fee() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let pallet_account = crate::Pallet::<Test>::account_id();
        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();
        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(10_000);
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &pallet_account),
            expected_fee,
            "fee hasnt been taken out of project as expected."
        );
    });
}

#[test]
fn withdraw_cannot_double_withdraw() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();
        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));
        assert_noop!(
            Proposals::withdraw(RuntimeOrigin::signed(ALICE), project_key),
            Error::<Test>::NoAvailableFundsToWithdraw
        );
    });
}

#[test]
fn withdraw_once_times_with_double_submissions() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, 0).unwrap();
        let _ =
            Proposals::vote_on_milestone(RuntimeOrigin::signed(BOB), project_key, 0, true).unwrap();
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, 1).unwrap();
        let _ =
            Proposals::vote_on_milestone(RuntimeOrigin::signed(BOB), project_key, 1, true).unwrap();

        let alice_before =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(20000);
        let alice_expected_balance = alice_before + 20000 - expected_fee;
        assert_eq!(
            alice_after, alice_expected_balance,
            "Alice account is not the expected balance"
        );
    });
}

// kind of a beast but worth it.
#[test]
fn withdraw_twice_with_intermitent_submission() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);

        // The first submission and withdraw
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, 0).unwrap();
        let _ =
            Proposals::vote_on_milestone(RuntimeOrigin::signed(BOB), project_key, 0, true).unwrap();
        let alice_before =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(10_000);
        let alice_expected_balance = alice_before + 10000 - expected_fee;
        assert_eq!(
            alice_after, alice_expected_balance,
            "Alice account is not the expected balance"
        );

        // The second submission and withdraw
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, 1).unwrap();
        let _ =
            Proposals::vote_on_milestone(RuntimeOrigin::signed(BOB), project_key, 1, true).unwrap();
        let alice_before =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(10000);
        let alice_expected_balance = alice_before + 10000 - expected_fee;
        assert_eq!(
            alice_after, alice_expected_balance,
            "Alice account is not the expected balance"
        );
    });
}

#[test]
fn withdraw_with_variable_percentage() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = vec![
            ProposedMilestone {
                percentage_to_unlock: Percent::from_percent(70u8),
            },
            ProposedMilestone {
                percentage_to_unlock: Percent::from_percent(30u8),
            },
        ];
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, 0).unwrap();
        let _ =
            Proposals::vote_on_milestone(RuntimeOrigin::signed(BOB), project_key, 0, true).unwrap();
        let alice_before =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(70000);
        let alice_expected_balance = alice_before + 70000 - expected_fee;
        assert_eq!(
            alice_after, alice_expected_balance,
            "Alice account is not the expected balance"
        );
    });
}

#[test]
fn withdraw_fails_before_approval() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_noop!(
            Proposals::withdraw(RuntimeOrigin::signed(ALICE), project_key),
            Error::<Test>::NoAvailableFundsToWithdraw
        );
        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();
        assert_noop!(
            Proposals::withdraw(RuntimeOrigin::signed(ALICE), project_key),
            Error::<Test>::NoAvailableFundsToWithdraw
        );
    });
}

#[test]
fn raise_no_confidence_round_already_started() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;

        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();
        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();
        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(BOB),
            project_key
        ));
        assert_noop!(
            Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(BOB), project_key),
            Error::<Test>::RoundStarted
        );
    });
}

#[test]
fn raise_no_confidence_round_not_contributor() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_noop!(
            Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(CHARLIE), project_key),
            Error::<Test>::OnlyContributorsCanVote
        );
    });
}

#[test]
fn raise_no_confidence_round_no_project() {
    build_test_externality().execute_with(|| {
        assert_noop!(
            Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(CHARLIE), 20),
            Error::<Test>::ProjectDoesNotExist
        );
    });
}

#[test]
fn raise_no_confidence_round_puts_initial_vote_is_isnay() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;

        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();
        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();

        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(BOB),
            project_key
        ));

        let vote = NoConfidenceVotes::<Test>::get(project_key).expect("vote should exist");
        assert_eq!(
            vote.nay, 100_000,
            "Bobs vote does not equal expected amount."
        );

        let has_voted = UserHasVoted::<Test>::get((project_key, RoundType::VoteOfNoConfidence, 0));
        assert!(
            has_voted.values().len() == 1usize,
            "The btree should only have a single value, the caller of the round."
        );
        assert!(
            has_voted.contains_key(&BOB),
            "Bob called the round so should be recorded as voted."
        );
    });
}

#[test]
fn vote_on_no_confidence_round_not_in_round() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);

        assert_noop!(
            Proposals::vote_on_no_confidence_round(
                RuntimeOrigin::signed(CHARLIE),
                project_key,
                true
            ),
            Error::<Test>::ProjectNotInRound
        );
    });
}

#[test]
fn vote_on_no_confidence_round_not_contributor() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);

        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(BOB),
            project_key
        ));
        assert_noop!(
            Proposals::vote_on_no_confidence_round(
                RuntimeOrigin::signed(CHARLIE),
                project_key,
                true
            ),
            Error::<Test>::OnlyContributorsCanVote
        );
    });
}

#[test]
fn vote_on_no_confidence_round_already_voted() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);

        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(BOB),
            project_key
        ));
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(DAVE),
            project_key,
            true
        ));
        assert_noop!(
            Proposals::vote_on_no_confidence_round(RuntimeOrigin::signed(DAVE), project_key, true),
            Error::<Test>::VotesAreImmutable
        );
    });
}

#[test]
fn vote_on_no_confidence_mutates_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);

        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(BOB),
            project_key
        ));
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(DAVE),
            project_key,
            true
        ));
        let vote = NoConfidenceVotes::<Test>::get(project_key).expect("vote should exist");
        assert_eq!(
            vote.nay, 100_000,
            "Total vote should equal half contributions here."
        );
        assert_eq!(
            vote.yay, 100_000,
            "Total vote should equal half contributions here."
        );

        let has_voted = UserHasVoted::<Test>::get((project_key, RoundType::VoteOfNoConfidence, 0));
        assert!(
            has_voted.values().len() == 2usize,
            "The btree should only have a single value, the caller of the round."
        );
        assert!(
            has_voted.contains_key(&BOB) && has_voted.contains_key(&DAVE),
            "Bob and charlie have voted."
        );
    });
}

#[test]
fn auto_finalizing_vote_on_no_confidence_when_threshold_is_met() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions::<Test>(vec![BOB, DAVE, CHARLIE, ALICE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project::<Test>(ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(BOB),
            project_key
        ));
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(DAVE),
            project_key,
            true
        ));
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(CHARLIE),
            project_key,
            false
        ));
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(ALICE),
            project_key,
            false
        ));
        let vote = NoConfidenceVotes::<Test>::get(project_key).expect("vote should exist");
        assert_eq!(
            vote.nay, 300_000,
            "Total vote should equal half contributions here."
        );
        assert_eq!(
            vote.yay, 100000,
            "Total vote should equal half contributions here."
        );

        let has_voted = UserHasVoted::<Test>::get((project_key, RoundType::VoteOfNoConfidence, 0));
        assert!(
            has_voted.values().len() == 4usize,
            "Not all the votes has been recorded"
        );
        assert!(
            has_voted.contains_key(&BOB)
                || has_voted.contains_key(&DAVE)
                || has_voted.contains_key(&ALICE)
                || has_voted.contains_key(&CHARLIE),
            "Bob,Alice,Dave charlie have voted."
        );
        assert_last_event::<Test>(
            Event::<Test>::NoConfidenceRoundFinalised(ALICE, project_key).into(),
        );
        assert_eq!(Projects::<Test>::get(project_key), None);
        assert_eq!(
            Rounds::<Test>::get((project_key, 0), RoundType::VoteOfNoConfidence),
            None
        );
    });
}

#[test]
fn close_voting_round_works() {
    build_test_externality().execute_with(|| {
        Rounds::<Test>::insert((0, 0), RoundType::VotingRound, 100);
        let r_expiring: BoundedVec<
            (ProjectKey, RoundType, MilestoneKey),
            <Test as Config>::ExpiringProjectRoundsPerBlock,
        > = vec![(0, RoundType::VotingRound, 0)]
            .try_into()
            .expect("smaller than bound: qed.");
        RoundsExpiring::<Test>::insert(100, r_expiring);

        let milestone_keys = vec![0];
        let mut i_v = ImmutableIndividualVotes::<Test>::new(milestone_keys.try_into().unwrap());
        assert_ok!(i_v.insert_individual_vote(0, &ALICE, true));

        IndividualVoteStore::<Test>::insert(0, i_v);
        assert_ok!(crate::Pallet::<Test>::close_voting_round(
            0,
            (0, RoundType::VotingRound, 0)
        ));

        assert!(Rounds::<Test>::get((0, 0), RoundType::VotingRound).is_none());
        assert!(RoundsExpiring::<Test>::get(100).len() == 0);
        let individual_votes = IndividualVoteStore::<Test>::get(0);
        assert!(individual_votes.is_some());
        assert!(individual_votes
            .unwrap()
            .as_ref()
            .get(&0)
            .unwrap()
            .is_empty());
    })
}

// todo: finalise voteof no confidence tests.
// ^^ is connected to making the pallet generic over funding type.
// Todo: assert the last event of each extrinsic/
