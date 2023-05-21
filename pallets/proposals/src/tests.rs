
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
        assert_noop!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), 0, 1), Error::<Test>::ProjectDoesNotExist);
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

        assert_eq!(created_vote.nay, 0);
        assert_eq!(created_vote.yay, 0);
    });
}

#[test]
fn submit_milestone_already_submitted() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1));
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 2));
        assert_noop!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 1), Error::<Test>::VoteAlreadyExists);
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
        assert!(false)
    });
}

#[test]
fn vote_on_milestone_already_voted() {
    build_test_externality().execute_with(|| {
        let cont = get_contributions(vec![*BOB, *CHARLIE], 100_000);
        let prop_milestones = get_milestones(10);
        let project_key = create_project(*ALICE, cont, prop_milestones, CurrencyId::Native);
        let milestone_key = 0;
        assert_ok!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key));
        assert_noop!(Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone_key), Error::<Test>::VoteAlreadyExists);
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
fn vote_on_milestone_doesnt_auto_finalise_below_threshold() {
    build_test_externality().execute_with(|| {
        assert!(false)
    });
}


#[test]
fn vote_on_milestone_does_auto_finalise_above_or_equal_threshold() {
    build_test_externality().execute_with(|| {
        assert!(false)
    });
}


fn get_contributions(accs: Vec<AccountId>, total_amount: Balance) -> ContributionsFor<Test> {
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
            percentage_to_unlock: 100u32 / n
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
fn create_project(
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