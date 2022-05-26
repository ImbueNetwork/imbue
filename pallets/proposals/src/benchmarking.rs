#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};

use crate::Pallet as Proposals;
use common_types::CurrencyId;
use frame_support::{
    traits::{Currency},
};
use sp_std::str;
use sp_std::vec::Vec;
use sp_runtime::traits::UniqueSaturatedFrom;


const CONTRIBUTION: u32 = 100;
const SEED: u32 = 0;

benchmarks! {
    where_clause { where
        T::AccountId: AsRef<[u8]>,
    }

    create_project{
        //let a in 1 .. 100;
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
        let project_keys: Vec<ProjectKey> = vec![0];

    }: _(RawOrigin::Root, start_block, end_block, project_keys.clone(), RoundType::ContributionRound)
    verify {
        assert_last_event::<T>(Event::FundingRoundCreated(0, project_keys).into());
    }

    cancel_round {
        
        let caller: T::AccountId = whitelisted_caller();
        //Setting the start block to be greater than 0 which is the current block. 
        //This condition is checked to ensure the round being cancelled has not started yet.
        //Benchmark seems to be starting at block 1, hence setting starting block to 2
        let start_block: T::BlockNumber = 2u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];
        
        //create project
        create_project_common::<T>(CONTRIBUTION);
        //schedule round
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), start_block, end_block, project_key, RoundType::ContributionRound)?;

    }: _(RawOrigin::Root, 0)
    verify {
       //assert_last_event::<T>(Event::RoundCancelled(0).into());
    }

    contribute {
        
        //create a funded user for contribution
        let alice: T::AccountId = create_funded_user::<T>("candidate", 1, 1000);

        
        //Setting the start block to be greater than 0 which is the current block. 
        //This condition is checked to ensure the round being cancelled has not started yet.
        //Benchmark seems to be starting at block 1, hence setting starting block to 2
        let start_block: T::BlockNumber = 2u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];
        let currency_id = CurrencyId::Native;
        let contribution_amount: BalanceOf<T> = BalanceOf::<T>::unique_saturated_from(1_000_000_000_000 as u128);
        let progress_block_number: <T as frame_system::Config>::BlockNumber = 3u32.into();
        
        //create project
        create_project_common::<T>(CONTRIBUTION);
        //schedule round
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), start_block, end_block, project_key, RoundType::ContributionRound)?;
        //progress the blocks
        run_to_block::<T>(progress_block_number);

    }: _(RawOrigin::Signed(alice.clone()), 0, contribution_amount)
    verify {
        //assert_last_event::<T>(Event::ContributeSucceeded(alice,0,contribution_amount,currency_id,progress_block_number).into());
    }

    approve {        
        //create a funded user for contribution
        let alice: T::AccountId = create_funded_user::<T>("candidate", 1, 1000);

        //Setting the start block to be greater than 0 which is the current block. 
        //This condition is checked to ensure the round being cancelled has not started yet.
        //Benchmark seems to be starting at block 1, hence setting starting block to 2
        let start_block: T::BlockNumber = 2u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];
        let currency_id = CurrencyId::Native;
        let contribution_amount: BalanceOf<T> = BalanceOf::<T>::unique_saturated_from(1_000_000_000_000 as u128);
        let milestone_keys: Vec<MilestoneKey> = vec![0];
        let progress_block_number: <T as frame_system::Config>::BlockNumber = 3u32.into();
        
        //create project
        create_project_common::<T>(CONTRIBUTION);
        //schedule round
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), start_block, end_block, project_key, RoundType::ContributionRound)?;
        //progress the blocks
        run_to_block::<T>(progress_block_number);
        //contribute
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), 0, contribution_amount)?;
        
        //2nd argument - project key
    }: _(RawOrigin::Root, 0, Some(milestone_keys))
    verify {
       //assert_last_event::<T>(Event::ProjectApproved(1,0).into());
    }

    submit_milestone { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 1000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1000);

        //Setting the start block to be greater than 0 which is the current block. 
        //This condition is checked to ensure the round being cancelled has not started yet.
        //Benchmark seems to be starting at block 1, hence setting starting block to 2
        let start_block: T::BlockNumber = 2u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];
        let currency_id = CurrencyId::Native;
        let contribution_amount: BalanceOf<T> = BalanceOf::<T>::unique_saturated_from(1_000_000_000_000 as u128);
        let milestone_keys: Vec<MilestoneKey> = vec![0];
        let progress_block_number: <T as frame_system::Config>::BlockNumber = 3u32.into();
        
        
        //create project
        create_project_common::<T>(CONTRIBUTION);
        //Proposals::<T>::create_project(RawOrigin::Signed(caller.clone()).into(), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id)?;
        //schedule round
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), start_block, end_block, project_key, RoundType::ContributionRound)?;
        //progress the blocks
        run_to_block::<T>(progress_block_number);
        //contribute
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), 0, contribution_amount)?;
        //Approve
        Proposals::<T>::approve(RawOrigin::Root.into(), 0, Some(milestone_keys))?;

        //project key - 2nd argument as u32 instead of vec
        //Milestone key - 3rd argument as u32
    }: _(RawOrigin::Signed(bob.clone()), 0, 0)
    verify {
       //assert_last_event::<T>(Event::VotingRoundCreated(1).into());
    }

    vote_on_milestone { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 1000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1000);

        //Setting the start block to be greater than 0 which is the current block. 
        //This condition is checked to ensure the round being cancelled has not started yet.
        //Benchmark seems to be starting at block 1, hence setting starting block to 2
        let start_block: T::BlockNumber = 2u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];
        let currency_id = CurrencyId::Native;
        let contribution_amount: BalanceOf<T> = BalanceOf::<T>::unique_saturated_from(1_000_000_000_000 as u128);
        let milestone_keys: Vec<MilestoneKey> = vec![0];
        let progress_block_number_contribute: <T as frame_system::Config>::BlockNumber = 3u32.into();
        let progress_block_number_vote_on_milestone: <T as frame_system::Config>::BlockNumber = 11u32.into();
        
        
        //create project
        create_project_common::<T>(CONTRIBUTION);
        //Proposals::<T>::create_project(RawOrigin::Signed(caller.clone()).into(), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id)?;
        //schedule round
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), start_block, end_block, project_key, RoundType::ContributionRound)?;
        //progress the blocks - to a block after the round start block for the project
        run_to_block::<T>(progress_block_number_contribute);
        //contribute
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), 0, contribution_amount)?;
        //Approve
        Proposals::<T>::approve(RawOrigin::Root.into(), 0, Some(milestone_keys))?;
        //Submit Milestone
        //project key - 2nd argument as u32 instead of vec
        //Milestone key - 3rd argument as u32
        Proposals::<T>::submit_milestone(RawOrigin::Signed(bob.clone()).into(), 0, 0)?;
        //progress the blocks - to a block after the round end block for the project
        run_to_block::<T>(progress_block_number_vote_on_milestone);

        //project key - 2nd argument as u32 instead of vec
        //Milestone key - 3rd argument as u32
        //approval boolean as approved - 4th argument
    }: _(RawOrigin::Signed(alice.clone()), 0, 0, true)
    verify {
        //assert_last_event::<T>(Event::VoteComplete(alice, 0, 0, true, progress_block_number_vote_on_milestone).into());
    }


    finalise_milestone_voting { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 1000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1000);

        //Setting the start block to be greater than 0 which is the current block. 
        //This condition is checked to ensure the round being cancelled has not started yet.
        //Benchmark seems to be starting at block 1, hence setting starting block to 2
        let start_block: T::BlockNumber = 2u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];
        let currency_id = CurrencyId::Native;
        let contribution_amount: BalanceOf<T> = BalanceOf::<T>::unique_saturated_from(1_000_000_000_000 as u128);
        let milestone_keys: Vec<MilestoneKey> = vec![0];
        let progress_block_number_contribute: <T as frame_system::Config>::BlockNumber = 3u32.into();
        let progress_block_number_vote_on_milestone: <T as frame_system::Config>::BlockNumber = 11u32.into();
        
        
        //create project
        create_project_common::<T>(CONTRIBUTION);
        //Proposals::<T>::create_project(RawOrigin::Signed(caller.clone()).into(), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id)?;
        //schedule round
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), start_block, end_block, project_key, RoundType::ContributionRound)?;
        //progress the blocks - to a block after the round start block for the project
        run_to_block::<T>(progress_block_number_contribute);
        //contribute
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), 0, contribution_amount)?;
        //Approve
        Proposals::<T>::approve(RawOrigin::Root.into(), 0, Some(milestone_keys))?;
        //Submit Milestone
        //project key - 2nd argument as u32 instead of vec
        //Milestone key - 3rd argument as u32
        Proposals::<T>::submit_milestone(RawOrigin::Signed(bob.clone()).into(), 0, 0)?;
        //progress the blocks - to a block after the round end block for the project
        run_to_block::<T>(progress_block_number_vote_on_milestone);
        //Vote on a milestone
        //project key - 2nd argument as u32 instead of vec
        //Milestone key - 3rd argument as u32
        //approval boolean as approved - 4th argument
        Proposals::<T>::vote_on_milestone(RawOrigin::Signed(alice.clone()).into(), 0, 0, true)?;

        //Finalization done by contributor in this case - 1st argument
        //project key - 2nd argument
        //milestone key - 3rd argument
    }: _(RawOrigin::Signed(bob.clone()), 0, 0)
    verify {
       //assert_last_event::<T>(Event::MilestoneApproved(0,0,progress_block_number_vote_on_milestone).into());
    }

      withdraw {
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 1000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1000);

        //Setting the start block to be greater than 0 which is the current block.
        //This condition is checked to ensure the round being cancelled has not started yet.
        //Benchmark seems to be starting at block 1, hence setting starting block to 2
        let start_block: T::BlockNumber = 0u32.into();
        let end_block: T::BlockNumber = 10u32.into();
        let project_key: Vec<ProjectKey> = vec![0];
        let currency_id = CurrencyId::Native;
        let contribution_amount: BalanceOf<T> = BalanceOf::<T>::unique_saturated_from(1_000_000_000_000 as u128);
        let milestone_keys: Vec<MilestoneKey> = vec![0];
        let progress_block_number_contribute: <T as frame_system::Config>::BlockNumber = 3u32.into();
        let progress_block_number_vote_on_milestone: <T as frame_system::Config>::BlockNumber = 5u32.into();
        let required_funds: BalanceOf<T> = 100u32.into();


        //create project
        create_project_common::<T>(CONTRIBUTION);
        //Proposals::<T>::create_project(RawOrigin::Signed(caller.clone()).into(), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id)?;
        //schedule round
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), start_block, end_block, project_key, RoundType::ContributionRound)?;
        //progress the blocks - to a block after the round start block for the project
        run_to_block::<T>(progress_block_number_contribute);
        //contribute
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), 0, contribution_amount)?;
        //Approve
        Proposals::<T>::approve(RawOrigin::Root.into(), 0, Some(milestone_keys))?;
        //Submit Milestone
        //project key - 2nd argument as u32 instead of vec
        //Milestone key - 3rd argument as u32
        Proposals::<T>::submit_milestone(RawOrigin::Signed(bob.clone()).into(), 0, 0)?;
        //progress the blocks - to a block after the round end block for the project
        run_to_block::<T>(progress_block_number_vote_on_milestone);
        //Vote on a milestone
        //project key - 2nd argument as u32 instead of vec
        //Milestone key - 3rd argument as u32
        //approval boolean as approved - 4th argumentEndBlockNumberInvalid
        Proposals::<T>::vote_on_milestone(RawOrigin::Signed(alice.clone()).into(), 0, 0, true)?;
        //Finalizing a milestone done
        //initiator or admin, in this case initiator - 1st argument
        //project key - 2nd argument
        //milestone key - 3rd argument
        Proposals::<T>::finalise_milestone_voting(RawOrigin::Signed(bob.clone()).into(),0,0)?;

        // Withdraw method takes the project initiator and the project id for which user wants to withdraw the funds for
    }: _(RawOrigin::Signed(bob.clone()), 0)
    verify {
        //assert_last_event::<T>(Event::ProjectFundsWithdrawn(bob,0,required_funds,currency_id).into());
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

fn create_project_common<T: Config>(contribution: u32) {
        let _caller: T::AccountId = whitelisted_caller();
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1000);
        let project_name: Vec<u8> = str::from_utf8(b"Imbue's Awesome Initiative").unwrap().as_bytes().to_vec();
        let project_logo: Vec<u8> = str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec();
        let project_description: Vec<u8> = str::from_utf8(b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.").unwrap().as_bytes().to_vec();
        let website: Vec<u8> = str::from_utf8(b"https://imbue.network").unwrap().as_bytes().to_vec();
        let milestones: Vec<ProposedMilestone> = vec![ProposedMilestone {
            name: Vec::new(),
            percentage_to_unlock: 100,
        }];

        let required_funds: BalanceOf<T> = contribution.into();
        let currency_id = CurrencyId::Native;
        
        let _start_block: T::BlockNumber = 0u32.into();

        let _ =Proposals::<T>::create_project(RawOrigin::Signed(bob.clone()).into(), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id);
        
}

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


impl_benchmark_test_suite!(Proposals, crate::mock::new_test_ext(), crate::mock::Test);
