#![cfg(feature = "runtime-benchmarks")]
use super::*;
use crate::Pallet as Proposals;
use common_types::CurrencyId;
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;

use sp_core::Get;
use sp_runtime::SaturatedConversion;
use sp_runtime::Saturating;
use sp_std::convert::TryInto;

use pallet_disputes::traits::DisputeHooks;
use pallet_disputes::DisputeResult;

use test_utils::{
    assert_last_event, create_and_fund_project, create_funded_user, get_contributions,
    get_max_milestones, get_milestones,
};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_milestone() {
        let alice: T::AccountId =
            create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let contributions = get_contributions::<T>(vec![alice], 100_000_000_000_000_000u128);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_and_fund_project::<T>(
            bob.clone(),
            contributions,
            prop_milestones,
            CurrencyId::Native,
        )
        .unwrap();

        #[extrinsic_call]
        submit_milestone(RawOrigin::Signed(bob), project_key, 0);
        assert_last_event::<T>(Event::<T>::VotingRoundCreated(project_key).into());
    }

    #[benchmark]
    fn vote_on_milestone() {
        let alice: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        // TODO: should update the contributors list to have maximum available length
        let contributions = get_contributions::<T>(vec![bob.clone()], 1_000_000_000_000u128);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_and_fund_project::<T>(
            alice.clone(),
            contributions,
            prop_milestones,
            CurrencyId::Native,
        )
        .unwrap();

        assert_ok!(Proposals::<T>::submit_milestone(
            RawOrigin::Signed(alice).into(),
            project_key,
            0
        ));

        #[extrinsic_call]
        vote_on_milestone(RawOrigin::Signed(bob.clone()), project_key, 0, true);
        let current_block = frame_system::Pallet::<T>::block_number();
        assert_last_event::<T>(
            Event::<T>::MilestoneApproved(bob, project_key, 0, current_block).into(),
        )
    }

    #[benchmark]
    fn withdraw() {
        let alice: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        let contributions = get_contributions::<T>(vec![bob.clone()], 100_000_000_000_000_000u128);
        let raised_funds = 100_000_000_000_000_000u128.saturated_into();

        let milestone_count = <T as Config>::MaxMilestonesPerProject::get();
        let prop_milestones = get_milestones(milestone_count as u8);

        let project_key = create_and_fund_project::<T>(
            alice.clone(),
            contributions,
            prop_milestones,
            CurrencyId::Native,
        )
        .unwrap();

        for milestone_key in 0..milestone_count {
            // The initiator submits a milestone
            assert_ok!(Proposals::<T>::submit_milestone(
                RawOrigin::Signed(alice.clone()).into(),
                project_key,
                milestone_key
            ));

            // Contributors vote on the milestone
            assert_ok!(Proposals::<T>::vote_on_milestone(
                RawOrigin::Signed(bob.clone()).into(),
                project_key,
                milestone_key,
                true
            ));
        }

        #[extrinsic_call]
        withdraw(RawOrigin::Signed(alice.clone()), project_key);
        assert_last_event::<T>(
            Event::<T>::ProjectFundsWithdrawn(alice, project_key, raised_funds, CurrencyId::Native)
                .into(),
        );
    }

    // Benchmark for a single loop of on_initialise as a voting round (most expensive).
    #[benchmark]
    fn on_initialize() {
        let block_number = 100u32.into();
        let keys: BoundedVec<
            (ProjectKey, RoundType, MilestoneKey),
            <T as Config>::ExpiringProjectRoundsPerBlock,
        > = vec![(0, RoundType::VotingRound, 0)]
            .try_into()
            .expect("bound will be larger than 1;");

        RoundsExpiring::<T>::insert(block_number, keys);
        #[block]
        {
            crate::Pallet::<T>::on_initialize(block_number);
        }
    }

    #[benchmark]
    fn raise_dispute() {
        let contribution_amount = 1_000_000_000_000u128;
        let alice: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("contributor", 0, 1_000_000_000_000_000_000u128);

        let contributors: Vec<T::AccountId> = (0
            ..<T as Config>::MaximumContributorsPerProject::get())
            .map(|i| create_funded_user::<T>("contributor", i, 1_000_000_000_000_000_000u128))
            .collect();

        let contributions = get_contributions::<T>(contributors, contribution_amount);
        let milestone_count = <T as Config>::MaxMilestonesPerProject::get();
        let prop_milestones = get_milestones(milestone_count as u8);
        let milestone_keys: BoundedVec<u32, <T as Config>::MaxMilestonesPerProject> = (0u32
            ..prop_milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();

        let project_key =
            create_and_fund_project::<T>(alice, contributions, prop_milestones, CurrencyId::Native)
                .unwrap();

        #[extrinsic_call]
        raise_dispute(RawOrigin::Signed(bob), project_key, milestone_keys);
    }

    #[benchmark]
    fn refund() {
        let contribution_amount = 1_000_000_000_000u128.saturated_into();
        let alice: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("contributor", 0, 1_000_000_000_000_000_000u128);

        let contributors: Vec<T::AccountId> = (0
            ..<T as Config>::MaximumContributorsPerProject::get())
            .map(|i| create_funded_user::<T>("contributor", i, 1_000_000_000_000_000_000u128))
            .collect();

        let contributions = get_contributions::<T>(contributors, contribution_amount);
        let total_amount =
            contribution_amount * <T as Config>::MaximumContributorsPerProject::get() as u128;
        let milestone_count = <T as Config>::MaxMilestonesPerProject::get();
        let prop_milestones = get_milestones(milestone_count as u8);
        let milestone_keys: BoundedVec<u32, <T as Config>::MaxMilestonesPerProject> = (0u32
            ..prop_milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();

        let project_key =
            create_and_fund_project::<T>(alice, contributions, prop_milestones, CurrencyId::Native)
                .unwrap();

        assert_ok!(crate::Pallet::<T>::raise_dispute(
            RawOrigin::Signed(bob.clone()).into(),
            project_key,
            milestone_keys.clone()
        ));
        let _ = <crate::Pallet<T> as DisputeHooks<ProjectKey, MilestoneKey>>::on_dispute_complete(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );

        #[extrinsic_call]
        refund(RawOrigin::Signed(bob), project_key);
        assert_last_event::<T>(
            Event::<T>::ProjectRefunded {
                project_key,
                total_amount: total_amount.saturated_into(),
            }
            .into(),
        );
    }

    impl_benchmark_test_suite!(
        Proposals,
        crate::mock::build_test_externality(),
        crate::mock::Test
    );
}
