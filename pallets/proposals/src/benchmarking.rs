#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, vec};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_core::Get;
use sp_runtime::SaturatedConversion;
use sp_runtime::Saturating;

use crate::Pallet as Proposals;
use crate::test_utils::*;

use super::*;

benchmarks! {
    where_clause {
        where T::AccountId: AsRef<[u8]>,
    }

    submit_milestone {
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let contributions = get_contributions::<T>(vec![alice], 100_000_000_000_000_000u128);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_project::<T>(bob.clone(), contributions, prop_milestones, CurrencyId::Native);
        // (Initiator, ProjectKey, MilestoneKey)
    }: _(RawOrigin::Signed(bob.clone()), project_key, 0)
    verify{
        assert_last_event::<T>(Event::<T>::VotingRoundCreated(project_key).into());
    }

    vote_on_milestone {
        let alice: T::AccountId = create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        // TODO: should update the contributors list to have maximum available length
        let contributions = get_contributions::<T>(vec![bob.clone()], 100_000_000_000_000_000u128);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_project::<T>(alice.clone(), contributions, prop_milestones, CurrencyId::Native);

        assert_ok!(Proposals::<T>::submit_milestone(RawOrigin::Signed(alice).into(), project_key, 0));
        // Contributor, ProjectKey, MilestoneKey, ApproveMilestone
    }: _(RawOrigin::Signed(bob.clone()), project_key, 0, true)
    verify {
        let current_block: T::BlockNumber = frame_system::Pallet::<T>::block_number();
        assert_last_event::<T>(Event::<T>::VoteSubmitted(bob, project_key, 0, true, current_block).into())
    }

    withdraw {
        let alice: T::AccountId = create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        let contributions = get_contributions::<T>(vec![bob.clone()], 100_000_000_000_000_000u128);
        let raised_funds:BalanceOf<T> = 100_000_000_000_000_000u128.saturated_into();

        let milestone_count = <T as Config>::MaxMilestonesPerProject::get();
        let prop_milestones = get_milestones(milestone_count as u8);

        let project_key = create_project::<T>(alice.clone(), contributions, prop_milestones, CurrencyId::Native);

        for milestone_key in 0..milestone_count {
            // The initiator submits a milestone
            assert_ok!(Proposals::<T>::submit_milestone(RawOrigin::Signed(alice.clone()).into(), project_key, milestone_key));

            // Contributors vote on the milestone
            assert_ok!(Proposals::<T>::vote_on_milestone(RawOrigin::Signed(bob.clone()).into(), project_key, milestone_key, true));
        }

        // All the milestones are approved now
        let fee:BalanceOf<T> = <T as Config>::ImbueFee::get().mul_floor(raised_funds).into();
        let withdrawn:BalanceOf<T> = raised_funds.saturating_sub(fee);

        // (Initiator, ProjectKey)
    }: _(RawOrigin::Signed(alice.clone()), project_key)
    verify {
        assert_last_event::<T>(Event::<T>::ProjectFundsWithdrawn(alice, project_key, withdrawn, CurrencyId::Native).into());
    }

    raise_vote_of_no_confidence {
        let alice: T::AccountId = create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        // TODO: should update the contributors list to have maximum available length
        let contributions = get_contributions::<T>(vec![bob.clone()], 100_000_000_000_000_000u128);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_project::<T>(alice.clone(), contributions, prop_milestones, CurrencyId::Native);

        // (Contributor, ProjectKey)
    }: _(RawOrigin::Signed(bob.clone()), project_key)
    verify {
        assert_last_event::<T>(Event::<T>::NoConfidenceRoundCreated(bob, project_key).into());
    }

    vote_on_no_confidence_round {
        let alice: T::AccountId = create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        let charlie: T::AccountId = create_funded_user::<T>("contributor", 2, 1_000_000_000_000_000_000u128);
        // TODO: should update the contributors list to have maximum available length
        let contributions = get_contributions::<T>(vec![bob.clone(), charlie.clone()], 100_000_000_000_000_000u128);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_project::<T>(alice.clone(), contributions, prop_milestones, CurrencyId::Native);

        assert_ok!(Pallet::<T>::raise_vote_of_no_confidence(RawOrigin::Signed(bob).into(), project_key));
        // (Contributor, ProjectKey, IsYay)
    }: _(RawOrigin::Signed(charlie.clone()), project_key, true)
    verify {
        assert_last_event::<T>(Event::<T>::NoConfidenceRoundVotedUpon(charlie, project_key).into());
    }

}

impl_benchmark_test_suite!(
    Proposals,
    crate::mock::build_test_externality(),
    crate::mock::Test
);
