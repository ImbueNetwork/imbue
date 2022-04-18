#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};

//System: frame_system::{Pallet, Call, Config, Storage, Event<T>},

use crate::Pallet as Proposals;
use codec::Encode;
use common_types::CurrencyId;
use frame_support::{
    assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo, weights::PostDispatchInfo,
};
use sp_std::str;
use sp_std::vec::Vec;
 
const CONTRIBUTION: u32 = 400;

benchmarks! {
    where_clause { where
        T::AccountId: AsRef<[u8]>,
    }

    create_project{
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let project_name: Vec<u8> = str::from_utf8(b"Imbue's Awesome Initiative").unwrap().as_bytes().to_vec();
        let project_logo: Vec<u8> = str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec();
        let project_description: Vec<u8> = str::from_utf8(b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.").unwrap().as_bytes().to_vec();
        let website: Vec<u8> = str::from_utf8(b"https://imbue.network").unwrap().as_bytes().to_vec();
        let milestones: Vec<ProposedMilestone> = vec![ProposedMilestone {
            name: Vec::new(),
            percentage_to_unlock: 100,
        }];

        let required_funds: BalanceOf<T> = 100u32.into();
        let currency_id = CurrencyId::Native;

    }: _(RawOrigin::Signed(caller.clone()), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id)
    verify {
        assert_last_event::<T>(Event::ProjectCreated(caller,project_name.clone(),0, required_funds, currency_id).into());
    }

    schedule_round {
        create_project_common::<T>(CONTRIBUTION);
        let start_block: T::BlockNumber = 0u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];

    }: _(RawOrigin::Root, start_block, end_block, project_key)
    verify {
        assert_last_event::<T>(Event::FundingRoundCreated(0).into());
    }

    cancel_round {
        
        let caller: T::AccountId = whitelisted_caller();
        //Setting the start block to be greater than 0 which is the current block. 
        //This condition is checked to ensure the round being cancelled has not started yet.
        let start_block: T::BlockNumber = 2u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];
        
        create_project_common::<T>(CONTRIBUTION);
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), start_block, end_block, project_key)?;

    }: _(RawOrigin::Root, 0)
    verify {
       // assert_last_event::<T>(Event::FundingRoundCreated(0).into());
    }


}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::TestExternalitiesBuilder::default().build(|| {}),
    crate::mock::MockRuntime,
);

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event)
where
    <T as frame_system::Config>::AccountId: AsRef<[u8]>,
{
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::Event = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

fn create_project_common<T: Config>(projectKey: u32){
        let caller: T::AccountId = whitelisted_caller();
        let project_name: Vec<u8> = str::from_utf8(b"Imbue's Awesome Initiative").unwrap().as_bytes().to_vec();
        let project_logo: Vec<u8> = str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec();
        let project_description: Vec<u8> = str::from_utf8(b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.").unwrap().as_bytes().to_vec();
        let website: Vec<u8> = str::from_utf8(b"https://imbue.network").unwrap().as_bytes().to_vec();
        let milestones: Vec<ProposedMilestone> = vec![ProposedMilestone {
            name: Vec::new(),
            percentage_to_unlock: 100,
        }];

        let required_funds: BalanceOf<T> = 100u32.into();
        let currency_id = CurrencyId::Native;
        
        let start_block: T::BlockNumber = 0u32.into();

        Proposals::<T>::create_project(RawOrigin::Signed(caller.clone()).into(), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id);
}

fn run_to_block(n: u64) {
    while <T as frame_system::Config::Pallet>::block_number() < n {
        if frame_system::Pallet::block_number() > 1 {
            Proposals::on_finalize(frame_system::Pallet::block_number());
            frame_system::Pallet::on_finalize(frame_system::Pallet::block_number());
        }
        frame_system::Pallet::set_block_number(frame_system::Pallet::block_number() + 1);
        frame_system::Pallet::on_initialize(frame_system::Pallet::block_number());
        Proposals::on_initialize(frame_system::Pallet::block_number());
    }
}


impl_benchmark_test_suite!(Proposals, crate::mock::new_test_ext(), crate::mock::Test);
