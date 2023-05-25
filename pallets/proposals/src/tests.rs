
use crate::{
    mock::*, 
    *,
};
use frame_support::{assert_noop, assert_ok};
use common_types::{CurrencyId, FundingType};
use orml_traits::{MultiReservableCurrency, MultiCurrency};
use sp_core::H256;

#[test]
fn submit_milestone_milestone_doesnt_exist() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_noop!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 11), Error::<Test>::MilestoneDoesNotExist);
    });
}

#[test]
fn submit_milestone_no_project() {
    build_test_externality().execute_with(|| {
        assert_noop!(
            Proposals::create_project(
                RuntimeOrigin::signed(*ALICE),
                gen_hash(1),
                bounded_vec![ProposedMilestone {
                    percentage_to_unlock: Percent::from_percent(99u8)
                }],
                //funds required
                1000000u64,
                CurrencyId::Native
            ),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::MilestonesTotalPercentageMustEqual100.into()
            }
        );
    });
}

#[test]
fn submit_milestone_not_initiator() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_noop!(Proposals::submit_milestone(RuntimeOrigin::signed(*BOB), project_key, 1), Error::<Test>::UserIsNotInitiator);
        assert_noop!(Proposals::submit_milestone(RuntimeOrigin::signed(*DAVE), project_key, 1), Error::<Test>::UserIsNotInitiator);
    });
}

#[test]
fn submit_milestones_too_many_this_block() {
    build_test_externality().execute_with(|| {
        let max = <Test as Config>::ExpiringProjectRoundsPerBlock::get();
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);

        (0..=max).for_each(|i| {
            let project_key = create_project(*ALICE, cont.clone(), prop_milestones.clone(), CurrencyId::Native);
            if i != max {
                assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1));                
            } else {
                assert_noop!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1), Error::<Test>::Overflow);                
            }
        })
    });
}

#[test]
fn submit_milestone_creates_non_bias_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1));
        let created_vote = MilestoneVotes::<Test>::get(project_key, 1).expect("should exist");

        assert_eq!(created_vote.nay, 0, "initial vote should be default");
        assert_eq!(created_vote.yay, 0, "initial vote should be default");
    });
}

#[test]
fn submit_milestone_can_resubmit_during_voting_round() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key));
        assert_ok!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true));
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key));
        let user_voted = UserHasVoted::<Test>::get((project_key, RoundType::VotingRound, milestone_key));
        dbg!(&user_voted);
        assert_eq!(user_voted.len(), 0usize, "User votes should be defaulted on resubmission.");
        let group_vote = MilestoneVotes::<Test>::get(project_key, milestone_key).expect("group vote should exist.");
        assert_eq!(group_vote, Default::default(), "Group vote should have defaulted on resubmission");
    });
}

#[test]
fn submit_milestone_can_submit_again_after_failed_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1));
        let expiry_block = frame_system::Pallet::<Test>::block_number() + <Test as Config>::MilestoneVotingWindow::get() as u64;
        run_to_block(expiry_block + 1);
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1));
    });
}

#[test]
fn submit_milestone_cannot_submit_again_after_success_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key));
        assert_ok!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true));
        assert_ok!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*CHARLIE), project_key, milestone_key, true));
        // The auto approval should have approved it here.
        let expiry_block = frame_system::Pallet::<Test>::block_number() + <Test as Config>::MilestoneVotingWindow::get() as u64;
        run_to_block(expiry_block + 1);
        assert_noop!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key), Error::<Test>::MilestoneAlreadyApproved);
    });
}

#[test]
fn vote_on_milestone_no_project() {
    build_test_externality().execute_with(|| {
        assert_noop!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*ALICE), 0, 0, true), Error::<Test>::ProjectDoesNotExist);
    });
}

#[test]
fn vote_on_milestone_before_round_starts_fails() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_noop!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true), Error::<Test>::VotingRoundNotStarted);
    });
}

#[test]
fn vote_on_milestone_after_round_end_fails() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let expiring_block = frame_system::Pallet::<Test>::block_number() + <Test as Config>::MilestoneVotingWindow::get();
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key));
        run_to_block(expiring_block);
        assert_noop!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true), Error::<Test>::VotingRoundNotStarted);
    });
}

#[test]
fn vote_on_milestone_where_voting_round_is_active_but_not_the_correct_milestone() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 0));
        assert_noop!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, 1, true), Error::<Test>::VotingRoundNotStarted);
    });
}

#[test]
fn vote_on_milestone_not_contributor() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key));
        assert_noop!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*DAVE), project_key, milestone_key, true), Error::<Test>::OnlyContributorsCanVote);
    });
}

#[test]
fn vote_on_milestone_actually_adds_to_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key));
        assert_ok!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true));
        let vote = MilestoneVotes::<Test>::get(project_key, milestone_key).expect("vote should exist");
        assert!(vote.yay == 50_000u64);
        assert!(vote.nay == 0u64);
        assert_ok!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*CHARLIE), project_key, milestone_key, false));
        let vote = MilestoneVotes::<Test>::get(project_key, milestone_key).expect("vote should exist");
        assert!(vote.yay == 50_000u64);
        assert!(vote.nay == 50_000u64);
    });
}

#[test]
fn withdraw_not_initiator() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key));
        assert_ok!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true));
        assert_ok!(Proposals::vote_on_milestone(RuntimeOrigin::signed(*CHARLIE), project_key, milestone_key, true));
        
        assert_noop!(Proposals::withdraw(RuntimeOrigin::signed(*BOB), project_key), Error::<Test>::UserIsNotInitiator);
        assert_noop!(Proposals::withdraw(RuntimeOrigin::signed(*DAVE), project_key), Error::<Test>::UserIsNotInitiator);
    });
}

#[test]
fn withdraw_only_transfers_approved_milestones() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*CHARLIE), project_key, milestone_key, true).unwrap();

        let alice_before = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        dbg!(&alice_before);
        assert_ok!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        dbg!(&alice_after);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(10000);
        dbg!(&expected_fee);
        let alice_expected_balance = alice_before + 10000 - expected_fee;
        assert_eq!(alice_after, alice_expected_balance, "Alice account is not the expected balance");

        let project_account = crate::Pallet::<Test>::project_account_id(project_key);
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &project_account), 90_000, "funds havent been taken out of project as expected.");
    });
}

#[test]
fn withdraw_removes_project_after_all_funds_taken() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB], 100_000);
        let prop_milestones = get_milestones(1);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true).unwrap();
        assert!(Projects::<Test>::get(project_key).is_some());
        assert_ok!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key));
        assert!(Projects::<Test>::get(project_key).is_none(), "Project should have been removed after funds withdrawn.")
    });
}

#[test]
fn withdraw_takes_imbue_fee() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let pallet_account = crate::Pallet::<Test>::account_id();
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true).unwrap();
        assert_ok!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key));
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(10_000);
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &pallet_account), expected_fee, "fee hasnt been taken out of project as expected.");
    });
}

#[test]
fn withdraw_cannot_double_withdraw() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, milestone_key, true).unwrap();
        assert_ok!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key));
        assert_noop!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key), Error::<Test>::NoAvailableFundsToWithdraw);
    });
}

#[test]
fn withdraw_once_times_with_double_submissions() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 0).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, 0, true).unwrap();
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, 1, true).unwrap();

        let alice_before = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(20000);
        let alice_expected_balance = alice_before + 20000 - expected_fee;
        assert_eq!(alice_after, alice_expected_balance, "Alice account is not the expected balance");
    });
}

// kind of a beast but worth it.
#[test]
fn withdraw_twice_with_intermitent_submission() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        
        // The first submission and withdraw
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 0).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, 0, true).unwrap();
        let alice_before = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(10_000);
        let alice_expected_balance = alice_before + 10000 - expected_fee;
        assert_eq!(alice_after, alice_expected_balance, "Alice account is not the expected balance");

        // The second submission and withdraw
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, 1, true).unwrap();
        let alice_before = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(10000);
        let alice_expected_balance = alice_before + 10000 - expected_fee;
        assert_eq!(alice_after, alice_expected_balance, "Alice account is not the expected balance");
    });
}

#[test]
fn withdraw_with_variable_percentage() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB], 100_000);
        let prop_milestones = vec![
            ProposedMilestone {
                percentage_to_unlock: Percent::from_percent(70u8)
            },
            ProposedMilestone {
                percentage_to_unlock: Percent::from_percent(30u8)
            },
        ];
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let pallet_account = crate::Pallet::<Test>::account_id();
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 0).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, 0, true).unwrap();
        let alice_before = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key));
        let alice_after = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let expected_fee = <Test as Config>::ImbueFee::get().mul_floor(70000);
        let alice_expected_balance = alice_before + 70000 - expected_fee;
        assert_eq!(alice_after, alice_expected_balance, "Alice account is not the expected balance");
    });
}

#[test]
fn withdraw_fails_before_approval() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        let pallet_account = crate::Pallet::<Test>::account_id();
        assert_noop!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key), Error::<Test>::NoAvailableFundsToWithdraw);
        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key).unwrap();
        assert_noop!(Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key), Error::<Test>::NoAvailableFundsToWithdraw);
    });
}

#[test]
fn raise_no_confidence_round_already_started() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;

        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 0).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, 0, true).unwrap();
        assert_ok!(Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*BOB), project_key));
        assert_noop!(Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*BOB), project_key), Error::<Test>::RoundStarted);
    });
}

#[test]
fn raise_no_confidence_round_not_contributor() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;

        assert_noop!(Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*CHARLIE), project_key), Error::<Test>::OnlyContributorsCanVote);
    });
}

#[test]
fn raise_no_confidence_round_no_project() {
    build_test_externality().execute_with(|| {
        assert_noop!(Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*CHARLIE), 20), Error::<Test>::ProjectDoesNotExist);
    });
}

#[test]
fn raise_no_confidence_round_puts_initial_vote_is_isnay() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;

        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 0).unwrap();
        let _ = Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), project_key, 0, true).unwrap();
        assert_ok!(Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*BOB), project_key));

        let vote = NoConfidenceVotes::<Test>::get(project_key).expect("vote should exist");
        assert_eq!(vote.nay, 50_000, "Bobs vote does not equal expected amount.");

        let has_voted = UserHasVoted::<Test>::get((project_key, RoundType::VoteOfNoConfidence, 0));
        assert!(has_voted.values().len() == 1usize, "The btree should only have a single value, the caller of the round.");
        assert!(has_voted.contains_key(&BOB), "Bob called the round so should be recorded as voted.");
    });
}

#[test]
fn vote_on_no_confidence_round_no_project() {
    build_test_externality().execute_with(|| {
        assert_noop!(Proposals::finalise_no_confidence_round(RuntimeOrigin::signed(*CHARLIE), 20), Error::<Test>::ProjectDoesNotExist);
    });
}

#[test]
fn vote_on_no_confidence_round_not_in_round() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);

        assert_noop!(Proposals::vote_on_no_confidence_round(RuntimeOrigin::signed(*CHARLIE), project_key, true), Error::<Test>::ProjectNotInRound);
    });
}

#[test]
fn vote_on_no_confidence_round_not_contributor() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);

        assert_ok!(Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*BOB), project_key));
        assert_noop!(Proposals::vote_on_no_confidence_round(RuntimeOrigin::signed(*CHARLIE), project_key, true), Error::<Test>::OnlyContributorsCanVote);
    });
}

#[test]
fn vote_on_no_confidence_round_already_voted() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);

        assert_ok!(Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*BOB), project_key));
        assert_ok!(Proposals::vote_on_no_confidence_round(RuntimeOrigin::signed(*DAVE), project_key, true));
        assert_noop!(Proposals::vote_on_no_confidence_round(RuntimeOrigin::signed(*DAVE), project_key, true), Error::<Test>::VotesAreImmutable);
    });
}

#[test]
fn vote_on_no_confidence_mutates_vote() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *DAVE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);

        let yes_contribution = PercentRequiredForVoteToPass::get().mul_floor(1_000_000u64);
        let no_contribution = Percent::one()
            .saturating_sub(PercentRequiredForVoteToPass::get())
            .mul_floor(1_000_000u64);

        let has_voted = UserHasVoted::<Test>::get((project_key, RoundType::VoteOfNoConfidence, 0));
        assert!(has_voted.values().len() == 2usize, "The btree should only have a single value, the caller of the round.");
        assert!(has_voted.contains_key(&BOB) && has_voted.contains_key(&DAVE), "Bob and charlie have voted.");
    }); 
}

// todo: finalise voteof no confidence tests.
// ^^ is connected to making the pallet generic over funding type.
// Todo: assert the last event of each extrinsic/

pub fn get_contributions(accs: Vec<AccountId>, total_amount: Balance) -> ContributionsFor<Test> {
    let v = total_amount / accs.len() as u64;
    let now = frame_system::Pallet::<Test>::block_number();
    let mut out = BTreeMap::new();
    accs.iter().for_each(|a| {
        let c = Contribution {
            value: v,
            timestamp: now,
        };
        out.insert(a.clone(), c);
    });
    out
}

pub fn get_milestones(mut n: u32) -> Vec<ProposedMilestone> {
    (0..n).map(|_| {
        ProposedMilestone {
            percentage_to_unlock: Percent::from_percent(100u8 / n as u8)
        }
    }).collect::<Vec<ProposedMilestone>>()
}

pub fn run_to_block(n: BlockNumber) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Proposals::on_initialize(System::block_number());
    }
}

/// Create a project for test purposes, this will not test the paths coming into this pallet via
/// the IntoProposal trait.
pub fn create_project(
    benificiary: AccountIdOf<Test>,
    contributions: ContributionsFor<Test>,
    proposed_milestones: Vec<ProposedMilestone>,
    currency_id: CurrencyId,
) -> ProjectKey
{       
    let aggrement_hash: H256 = Default::default();
        let project_key = crate::ProjectCount::<Test>::get().saturating_add(1);
        crate::ProjectCount::<Test>::put(project_key);
        let sum_of_contributions = contributions
            .values()
            .fold(Default::default(), |acc: BalanceOf<Test>, x| {
                acc.saturating_add(x.value)
            });

        for (acc, cont) in contributions.iter() {
            let project_account_id = crate::Pallet::<Test>::project_account_id(project_key);
            assert_ok!(<Test as crate::Config>::MultiCurrency::transfer(
                RuntimeOrigin::signed(*acc),
                project_account_id,
                currency_id,
                cont.value,
            ));
        }

        let mut milestone_key: u32 = 0;
        let mut milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();
        for milestone in proposed_milestones {
            let milestone = Milestone {
                project_key,
                milestone_key,
                percentage_to_unlock: milestone.percentage_to_unlock,
                is_approved: false,
            };
            milestones.insert(milestone_key, milestone);
            milestone_key = milestone_key.saturating_add(1);
        }

        let project: Project<AccountIdOf<Test>, BalanceOf<Test>, BlockNumberFor<Test>> =
            Project {
                milestones,
                contributions,
                currency_id,
                withdrawn_funds: 0u32.into(),
                raised_funds: sum_of_contributions,
                initiator: benificiary.clone(),
                created_on: frame_system::Pallet::<Test>::block_number(),
                cancelled: false,
                agreement_hash: aggrement_hash,
                funding_type: FundingType::Brief,
            };

        Projects::<Test>::insert(project_key, project);
        let project_account = crate::Pallet::<Test>::project_account_id(project_key);
        ProjectCount::<Test>::mutate(|c| *c = c.saturating_add(1));
        project_key
}
