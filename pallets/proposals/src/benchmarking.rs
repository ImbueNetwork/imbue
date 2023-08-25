#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::test_utils::*;
use crate::Pallet as Proposals;
use common_types::{CurrencyId, TreasuryOrigin};
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_arithmetic::per_things::Percent;
use sp_core::Get;
use sp_runtime::SaturatedConversion;
use sp_runtime::Saturating;
use sp_std::{convert::TryInto, str, vec::Vec};

#[benchmarks( where
    <T as frame_system::Config>::AccountId: AsRef<[u8]>,
    <T as frame_system::Config>::BlockNumber: From<u32>,
)]

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
        let project_key = create_project::<T>(
            bob.clone(),
            contributions,
            prop_milestones,
            CurrencyId::Native,
        );

        #[extrinsic_call]
        submit_milestone(RawOrigin::Signed(bob.clone()), project_key, 0);
        assert_last_event::<T>(Event::<T>::VotingRoundCreated(project_key).into());
    }

    #[benchmark]
    fn vote_on_milestone() {
        let alice: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        // TODO: should update the contributors list to have maximum available length
        let contributions = get_contributions::<T>(vec![bob.clone()], 100_000_000_000_000_000u128);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_project::<T>(
            alice.clone(),
    z        contributions,
            prop_milestones,
            CurrencyId::Native,
        );

        assert_ok!(Proposals::<T>::submit_milestone(
            RawOrigin::Signed(alice).into(),
            project_key,
            0
        ));

        #[extrinsic_call]
        vote_on_milestone(RawOrigin::Signed(bob.clone()), project_key, 0, true);
        let current_block: T::BlockNumber = frame_system::Pallet::<T>::block_number();
        assert_last_event::<T>(
            Event::<T>::VoteSubmitted(bob, project_key, 0, true, current_block).into(),
        )
    }

    #[benchmark]
    fn withdraw() {
        let alice: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        let contributions = get_contributions::<T>(vec![bob.clone()], 100_000_000_000_000_000u128);
        let raised_funds: BalanceOf<T> = 100_000_000_000_000_000u128.saturated_into();

        let milestone_count = <T as Config>::MaxMilestonesPerProject::get();
        let prop_milestones = get_milestones(milestone_count as u8);

        let project_key = create_project::<T>(
            alice.clone(),
            contributions,
            prop_milestones,
            CurrencyId::Native,
        );

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

        // All the milestones are approved now
        let fee: BalanceOf<T> = <T as Config>::ImbueFee::get()
            .mul_floor(raised_funds)
            .into();
        let withdrawn: BalanceOf<T> = raised_funds.saturating_sub(fee);

        #[extrinsic_call]
        withdraw(RawOrigin::Signed(alice.clone()), project_key);
        assert_last_event::<T>(
            Event::<T>::ProjectFundsWithdrawn(alice, project_key, withdrawn, CurrencyId::Native)
                .into(),
        );
    }

    #[benchmark]
    fn raise_vote_of_no_confidence() {
        let alice: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        // TODO: should update the contributors list to have maximum available length
        let contributions = get_contributions::<T>(vec![bob.clone()], 100_000_000_000_000_000u128);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_project::<T>(
            alice.clone(),
            contributions,
            prop_milestones,
            CurrencyId::Native,
        );
        #[extrinsic_call]
        raise_vote_of_no_confidence(RawOrigin::Signed(bob.clone()), project_key);
        assert_last_event::<T>(Event::<T>::NoConfidenceRoundCreated(bob, project_key).into());
    }

    #[benchmark]
    fn vote_on_no_confidence_round() {
        let alice: T::AccountId =
            create_funded_user::<T>("initiator", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId =
            create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
        let charlie: T::AccountId =
            create_funded_user::<T>("contributor", 2, 1_000_000_000_000_000_000u128);
        // TODO: should update the contributors list to have maximum available length
        let contributions = get_contributions::<T>(
            vec![bob.clone(), charlie.clone()],
            100_000_000_000_000_000u128,
        );
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_project::<T>(
            alice.clone(),
            contributions,
            prop_milestones,
            CurrencyId::Native,
        );

        assert_ok!(Pallet::<T>::raise_vote_of_no_confidence(
            RawOrigin::Signed(bob).into(),
            project_key
        ));

        #[extrinsic_call]
        vote_on_no_confidence_round(RawOrigin::Signed(charlie.clone()), project_key, true);
        assert_last_event::<T>(Event::<T>::NoConfidenceRoundVotedUpon(charlie, project_key).into());
    }

    // Benchmark for a single loop of on_initialise as a voting round (most expensive).
    #[benchmark]
    fn on_initialize() {
        let block_number: <T as frame_system::Config>::BlockNumber = 100u32.into();
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

    impl_benchmark_test_suite!(
        Proposals,
        crate::mock::build_test_externality(),
        crate::mock::Test
    );
}
