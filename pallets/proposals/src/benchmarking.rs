#![cfg(feature = "runtime-benchmarks")]
use super::*;
use crate::test_utils::*;
use crate::Pallet as Proposals;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, vec};
use frame_system::RawOrigin;

benchmarks! {
    where_clause {
        where T::AccountId: AsRef<[u8]>,
    }

    submit_milestone {
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 1_000_000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1_000);
        let contributions = get_contributions::<T>(vec![alice], 100_000);
        let prop_milestones = get_max_milestones::<T>();
        let project_key = create_project::<T>(bob.clone(), contributions, prop_milestones, CurrencyId::Native);
        // (Initiator, ProjectKey, MilestoneKey)
    }: _(RawOrigin::Signed(bob.clone()), project_key, 0)
    verify{
        assert_last_event::<T>(Event::<T>::VotingRoundCreated(project_key).into());
    }
}

impl_benchmark_test_suite!(
    Proposals,
    crate::mock::build_test_externality(),
    crate::mock::Test
);
