#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};

use crate::Pallet as Proposals;
use common_types::CurrencyId;
use frame_support::{
    traits::{Currency, Get},
};
use sp_std::str;
use sp_std::vec::Vec;
use sp_runtime::traits::UniqueSaturatedFrom;

const CONTRIBUTION: u32 = 100;
const SEED: u32 = 0;


//accumulate_dummy {
//    let b in 1 .. 1000;
//    let caller = account("caller", 0, 0);
//  }: _ (RawOrigin::Signed(caller), b.into())
//

benchmarks! {
    where_clause { where
        T::AccountId: AsRef<[u8]>,
    }

    create_project {
        let caller: T::AccountId = whitelisted_caller();

        let bounded_str_f: BoundedStringField = "a".repeat(<MaxStringFieldLen as Get<u32>>::get() as usize).as_bytes().to_vec().try_into().unwrap();
        
        let bounded_desc_f: BoundedDescriptionField = "b".repeat(<MaxDescriptionField as Get<u32>>::get() as usize).as_bytes().to_vec().try_into().unwrap();
        
        let milestones: BoundedProposedMilestones = vec![ProposedMilestone {
            name: bounded_str_f.clone(),
            percentage_to_unlock: 1,
        }; 100].try_into().unwrap();

        let required_funds: BalanceOf<T> = u32::MAX.into();
        let currency_id = CurrencyId::Native;
        let caller: T::AccountId = whitelisted_caller();

    }: _(RawOrigin::Signed(caller.clone()), bounded_str_f.clone(), bounded_str_f.clone(), bounded_desc_f.clone(), bounded_desc_f.clone(), milestones, required_funds, CurrencyId::Native)
    verify {
        assert_last_event::<T>(Event::ProjectCreated(caller, bounded_str_f.clone().to_vec(), 0, required_funds, CurrencyId::Native).into());
    }
}

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
//
//fn create_project_common<T: Config>(contribution: u32) {
//        let _caller: T::AccountId = whitelisted_caller();
//        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1000);
//        let project_name: BoundedStringField = str::from_utf8(b"Imbue's Awesome Initiative").unwrap().as_bytes().to_vec();
//        let project_logo: BoundedStringField = str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec();
//        let project_description: BoundedDescriptionField<u8> = str::from_utf8(b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.").unwrap().as_bytes().to_vec();
//        let website: BoundedStringField = str::from_utf8(b"https://imbue.network").unwrap().as_bytes().to_vec();
//        let milestones: Vec<ProposedMilestone> = vec![ProposedMilestone {
//            name: Vec::new(),
//            percentage_to_unlock: 100,
//        }];
//
//        let required_funds: BalanceOf<T> = contribution.into();
//        let currency_id = CurrencyId::Native;
//        
//        let _start_block: T::BlockNumber = 0u32.into();
//
//        let _ =Proposals::<T>::create_project(RawOrigin::Signed(bob.clone()).into(), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id);
//        
//}
//
fn run_to_block<T: Config>(new_block: <T as frame_system::Config>::BlockNumber) {
    frame_system::Pallet::<T>::set_block_number(new_block);
}

fn create_funded_user<T: Config>(
	string: &'static str,
	n: u32,
	balance_factor: u32,
) -> T::AccountId {
	let user = account(string, n, SEED);
	let balance = T::Currency::minimum_balance() * balance_factor.into();
	let _ = T::Currency::make_free_balance_be(&user, balance);
	user
}

impl_benchmark_test_suite!(Proposals, crate::mock::build_test_externality(), crate::mock::Test);
