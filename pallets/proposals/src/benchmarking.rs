#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};
use crate::Pallet as Proposals;
use frame_support::{
    assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo, weights::PostDispatchInfo,
};
use sp_std::str;
use sp_std::vec::Vec;
use common_types::CurrencyId;

benchmarks! {
    where_clause { where
		T::AccountId: AsRef<[u8]>,
	}

    create_project {
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let project_name: Vec<u8> = str::from_utf8(b"Imbue's Awesome Initiative").unwrap().as_bytes().to_vec();
        let project_logo: Vec<u8> = str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec();
        let project_description: Vec<u8> = str::from_utf8(b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.").unwrap().as_bytes().to_vec();
        let website: Vec<u8> = str::from_utf8(b"https://imbue.network").unwrap().as_bytes().to_vec();
        let proposed_milestones: Vec<ProposedMilestone> = vec![ProposedMilestone {
            name: Vec::new(),
            percentage_to_unlock: 100,
        }];

        let required_funds: BalanceOf<T> = 100u32.into();
        let currency_id = CurrencyId::Native;

    }: _(RawOrigin::Signed(caller.clone()), project_name.clone(), project_logo, project_description, website, proposed_milestones, required_funds, currency_id)
    verify {
        assert_last_event::<T>(Event::ProjectCreated(caller,project_name.clone(),0, required_funds, currency_id).into());
    }
}

impl_benchmark_test_suite!(
	Pallet,
	crate::mock::TestExternalitiesBuilder::default().build(|| {}),
	crate::mock::MockRuntime,
);

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) where <T as frame_system::Config>::AccountId: AsRef<[u8]> {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}


impl_benchmark_test_suite!(Proposals, crate::mock::new_test_ext(), crate::mock::Test);