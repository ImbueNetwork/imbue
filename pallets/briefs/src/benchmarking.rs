#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::mock::{get_brief_owners, get_milestones};
use crate::Pallet as Briefs;
use crate::*;
use common_types::CurrencyId;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{assert_ok, traits::Get};
use frame_system::{EventRecord, Pallet as System, RawOrigin};
use sp_core::H256;
use sp_std::str;

const SEED: u32 = 0;

benchmarks! {
    where_clause {
        where T::AccountId: AsRef<[u8]>,
    }

    add_to_fellowship {
        let account_id = create_account_id::<T>("user", 1);
        // (origin, account_id)
    }: _(RawOrigin::Root, account_id.clone())
    verify {
        assert_last_event::<T>(Event::<T>::AccountApproved(account_id).into());
    }

    // create_brief {
    //     let caller: T::AccountId = whitelisted_caller();
    //     let max_brief_owners: u32 = <T as Config>::MaxBriefOwners::get();
    //     let brief_owners = get_brief_owners(max_brief_owners);
    //     let applicant = create_account_id::<T>("applicant", 1);
    //     let budget = 1_000_000u32.into();
    //     let initial_contribution = 100_000u32.into();
    //     let brief_id = H256::from([0; 32]);
    //     // let milestones = bounded_vec![].into();
    // }: _(RawOrigin::Signed(caller.clone()), Default::default(), applicant, budget, initial_contribution, brief_id, CurrencyId::Native, Default::default())
    // verify {
    // }
}

fn create_account_id<T: Config>(suri: &'static str, n: u32) -> T::AccountId {
    account(suri, n, SEED)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent)
where
    <T as frame_system::Config>::AccountId: AsRef<[u8]>,
{
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

impl_benchmark_test_suite!(
    Briefs,
    crate::mock::build_test_externality(),
    crate::mock::Test
);
