#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{Pallet as System, EventRecord, RawOrigin};
use crate::Pallet as Proposals;
use common_types::CurrencyId;
use frame_support::{
    assert_ok,
    traits::{Currency, Get},
};
use sp_std::str;
const _CONTRIBUTION: u32 = 100;
const SEED: u32 = 0;
//accumulate_dummy {
//    let b in 1 .. 1000;
//    let caller = account("caller", 0, 0);
//  }: _ (RawOrigin::Signed(caller), b.into())
//

benchmarks! {
    where_clause { 
        where
        T::AccountId: AsRef<[u8]>,
    }

    update_project {
        let bounded_str_f: BoundedStringField = "a".repeat(<MaxStringFieldLen as Get<u32>>::get() as usize).as_bytes().to_vec().try_into().unwrap();
        
        let bounded_desc_f: BoundedDescriptionField = "b".repeat(<MaxDescriptionField as Get<u32>>::get() as usize).as_bytes().to_vec().try_into().unwrap();
        
        let bounded_website_f: BoundedWebsiteUrlField = "c".repeat(<MaxWebsiteUrlField as Get<u32>>::get() as usize).as_bytes().to_vec().try_into().unwrap();
        let milestones: BoundedProposedMilestones = vec![ProposedMilestone {
            name: bounded_str_f.clone(),
            percentage_to_unlock: 1,
        }; 100].try_into().unwrap();
        
        let caller = create_project_common::<T>(u32::MAX.into());

        let required_funds: BalanceOf<T> = u32::MAX.into();
        let currency_id = CurrencyId::Native;
        //origin, project_key, name, logo, description, website, proposed_milestones, required_funds, currency_id
    }: _(RawOrigin::Signed(caller.clone()), 0, bounded_str_f.clone(), bounded_str_f.clone(), bounded_desc_f, bounded_website_f, milestones, required_funds, currency_id)
    verify {
        assert_last_event::<T>(Event::ProjectUpdated(caller, bounded_str_f.to_vec(), 0, required_funds).into());
    }

    create_project {
        let caller: T::AccountId = whitelisted_caller();

        let bounded_str_f: BoundedStringField = "a".repeat(<MaxStringFieldLen as Get<u32>>::get() as usize).as_bytes().to_vec().try_into().unwrap();
        
        let bounded_desc_f: BoundedDescriptionField = "b".repeat(<MaxDescriptionField as Get<u32>>::get() as usize).as_bytes().to_vec().try_into().unwrap();
        
        let bounded_website_f: BoundedWebsiteUrlField = "c".repeat(<MaxWebsiteUrlField as Get<u32>>::get() as usize).as_bytes().to_vec().try_into().unwrap();
        let milestones: BoundedProposedMilestones = vec![ProposedMilestone {
            name: bounded_str_f.clone(),
            percentage_to_unlock: 1,
        }; 100].try_into().unwrap();

        let required_funds: BalanceOf<T> = u32::MAX.into();
        let currency_id = CurrencyId::Native;

        // (Origin, ProjectName, Logo, Description, Website, ProposedMilestones, RequiredFunds, CurrencyId)
    }: _(RawOrigin::Signed(whitelisted_caller()), bounded_str_f.clone(), bounded_str_f.clone(), bounded_desc_f, bounded_website_f, milestones, required_funds, CurrencyId::Native)
    verify {
        assert_last_event::<T>(Event::ProjectCreated(caller, bounded_str_f.to_vec(), 0, required_funds, CurrencyId::Native).into());
    }

    add_project_whitelist {
        let caller = create_project_common::<T>(u32::MAX.into());
        let mut bbt : BoundedWhitelistSpots<T> = BTreeMap::new().try_into().unwrap();

        for i in 0..<MaxWhitelistPerProject as Get<u32>>::get() {
            bbt.try_insert(whitelisted_caller(), 100u32.into()).unwrap();
        }
        // (Origin, ProjectKey, BoundedWhitelistSpots)
    }: _(RawOrigin::Signed(caller), 0, bbt)
    verify {
        assert_last_event::<T>(Event::WhitelistAdded(0, 1u32.into()).into());
    }

    remove_project_whitelist {
        let caller = create_project_common::<T>(u32::MAX.into());
        let mut bbt : BoundedWhitelistSpots<T> = BTreeMap::new().try_into().unwrap();
        
        for i in 0..<MaxWhitelistPerProject as Get<u32>>::get() {
            bbt.try_insert(whitelisted_caller(), 100u32.into()).unwrap();
        }
        let _ = Proposals::<T>::add_project_whitelist(RawOrigin::Signed(caller.clone()).into(), 0, bbt);

        // (Origin, ProjectKey)
    }: _(RawOrigin::Signed(caller), 0u32)
    verify {
        assert_last_event::<T>(Event::WhitelistRemoved(0, 1u32.into()).into());
    }

    schedule_round {
        let mut project_keys: BoundedProjectKeys = vec![].try_into().unwrap();

        for i in 0..<MaxProjectKeysPerRound as Get<u32>>::get() {
            let _caller = create_project_common::<T>(u32::MAX.into());
            let _ = project_keys.try_push(i).unwrap();
        }
        
        // (Origin, StartBlockNumber, EndBlockNumber, ProjectKeys, RoundType)c
    }: _(RawOrigin::Root, 2u32.into(), 100u32.into(), project_keys.clone(), RoundType::ContributionRound)
    verify {
        assert_last_event::<T>(Event::FundingRoundCreated(1, project_keys.to_vec()).into());
    }

    cancel_round {
        let caller: T::AccountId = whitelisted_caller();
        let mut project_keys: BoundedProjectKeys = vec![].try_into().unwrap();
        for i in 0..<MaxProjectKeysPerRound as Get<u32>>::get() {
            let _caller = create_project_common::<T>(u32::MAX.into());
            let _ = project_keys.try_push(i).unwrap();
        }
        let _ = Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), project_keys, RoundType::ContributionRound);
    
        // Round key starts at 1
        //(Origin, RoundKey)
    }: _(RawOrigin::Root, 1)
    verify {
       assert_last_event::<T>(Event::RoundCancelled(1).into());
    }

    contribute {
        // Setup state.
        let a in 0 .. <MaxProjectKeysPerRound as Get<u32>>::get() - 1;
        let alice: T::AccountId = create_funded_user::<T>("candidate", 1, 100_000);
        let caller: T::AccountId = whitelisted_caller();
        let mut project_keys: BoundedProjectKeys = vec![].try_into().unwrap();
        for i in 0..<MaxProjectKeysPerRound as Get<u32>>::get() {
            let _caller = create_project_common::<T>(u32::MAX.into());
            let _ = project_keys.try_push(i).unwrap();
        }
        let _ = Proposals::<T>::schedule_round(RawOrigin::Root.into(), 3u32.into(), 10u32.into(), project_keys, RoundType::ContributionRound);
        
        // Progress the blocks to allow contribution.
        run_to_block::<T>(5u32.into());

        //(Origin, RoundKey, ProjectKey, Contribution)
    }: _(RawOrigin::Signed(alice.clone()), Some(1u32), a.into(), 10_000u32.into())
    verify {
        assert_last_event::<T>(Event::ContributeSucceeded(alice, a.into(), 10_000u32.into(), CurrencyId::Native, 5u32.into()).into());
    }

    approve {        
        // Setup state.
        let a in 0 .. <MaxProjectKeysPerRound as Get<u32>>::get() - 1;
        //create a funded user for contribution
        let contribution = 100_000u32;
        let alice: T::AccountId = create_funded_user::<T>("candidate", 1, contribution);
        let mut project_keys: BoundedProjectKeys = vec![].try_into().unwrap();
        for i in 0..<MaxProjectKeysPerRound as Get<u32>>::get() {
            let _caller = create_project_common::<T>(100_000u32.into());
            let _ = project_keys.try_push(i).unwrap();
        }
        let milestone_keys: BoundedMilestoneKeys = (0.. <MaxProposedMilestones as Get<u32>>::get()).collect::<Vec<u32>>().try_into().unwrap();
        let _ = Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), project_keys, RoundType::ContributionRound);
        run_to_block::<T>(5u32.into());
        let _ = Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), Some(1u32), a.into(), contribution.into());
        
        //(Origin, RoundKey, ProjectKey, MilestoneKeys)
    }: _(RawOrigin::Root, Some(1), a.into(), Some(milestone_keys))
    verify {
       assert_last_event::<T>(Event::ProjectApproved(1, a.into()).into());
    }

    submit_milestone { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 1_000_000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1000);

        let contribution_amount = 1_000_000u32;
        let milestone_keys: BoundedMilestoneKeys = vec![0].try_into().unwrap();

        // Setup state.
        create_project_common::<T>(contribution_amount.into());
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), vec![0u32].try_into().unwrap(), RoundType::ContributionRound)?;
        run_to_block::<T>(5u32.into());
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), Some(1), 0, contribution_amount.into())?;
        Proposals::<T>::approve(RawOrigin::Root.into(), Some(1), 0, Some(milestone_keys))?;

        // (Initiator, ProjectKey, MilestoneKey)
    }: _(RawOrigin::Signed(bob.clone()), 0, 0)
    verify {
       assert_last_event::<T>(Event::VotingRoundCreated(2, vec![0]).into());
    }

    vote_on_milestone { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 100_000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 100_000);

        let contribution_amount = 10_000u32;
        let milestone_keys: BoundedMilestoneKeys = vec![0].try_into().unwrap();

        // Setup state.
        create_project_common::<T>(contribution_amount.into());
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), vec![0u32].try_into().unwrap(), RoundType::ContributionRound)?;
        run_to_block::<T>(5u32.into());
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), Some(1), 0, contribution_amount.into())?;
        Proposals::<T>::approve(RawOrigin::Root.into(), Some(1), 0, Some(milestone_keys))?;
        Proposals::<T>::submit_milestone(RawOrigin::Signed(bob.clone()).into(), 0, 0)?;
        
        run_to_block::<T>(11u32.into());
        // (Voter, ProjectKey, MilestoneKey, Option<RoundKey>, is_approved)
    }: _(RawOrigin::Signed(alice.clone()), 0, 0, Some(2), true)
    verify {
        assert_last_event::<T>(Event::VoteComplete(alice, 0, 0, true, 11u32.into()).into());
    }

    finalise_milestone_voting { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 100_000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 100_000);

        let contribution_amount = 10_000u32;
        let milestone_keys: BoundedMilestoneKeys = vec![0].try_into().unwrap();

        // Setup state.
        create_project_common::<T>(contribution_amount.into());
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), vec![0u32].try_into().unwrap(), RoundType::ContributionRound)?;
        run_to_block::<T>(5u32.into());
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), Some(1), 0, contribution_amount.into())?;
        Proposals::<T>::approve(RawOrigin::Root.into(), Some(1), 0, Some(milestone_keys))?;
        Proposals::<T>::submit_milestone(RawOrigin::Signed(bob.clone()).into(), 0, 0)?;
        run_to_block::<T>(11u32.into());
        Proposals::<T>::vote_on_milestone(RawOrigin::Signed(alice.clone()).into(), 0, 0, Some(2), true)?;
        
        // (Initiator, ProjectKey, MilestoneKey)
    }: _(RawOrigin::Signed(bob.clone()), 0, 0)
    verify {
        assert_last_event::<T>(Event::MilestoneApproved(bob, 0, 0, 11u32.into()).into());
    }

    withdraw { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 10_000_000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 100_000);

        let contribution_amount = 10_000u32;
        let milestone_keys: BoundedMilestoneKeys = (0..<MaxMilestoneKeys as Get<u32>>::get()).collect::<Vec<MilestoneKey>>().try_into().unwrap();
        
        // Setup state.
        create_project_common::<T>(contribution_amount.into());
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), vec![0].try_into().unwrap(), RoundType::ContributionRound)?;
        run_to_block::<T>(5u32.into());
        for key in milestone_keys.clone() {
            Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), Some(1), 0, contribution_amount.into())?;
        }
        Proposals::<T>::approve(RawOrigin::Root.into(), Some(1), 0, Some(milestone_keys.clone()))?;
        
        for key in milestone_keys.clone() {
            Proposals::<T>::submit_milestone(RawOrigin::Signed(bob.clone()).into(), 0, key)?;
            run_to_block::<T>(System::<T>::block_number() + 1u32.into());
            Proposals::<T>::vote_on_milestone(RawOrigin::Signed(alice.clone()).into(), 0, key, Some(key + 2u32), true)?;
            Proposals::<T>::finalise_milestone_voting(RawOrigin::Signed(bob.clone()).into(), 0, key)?;
        }
        
        // (Initiator, ProjectKey)
    }: _(RawOrigin::Signed(bob.clone()) ,0)
    verify {
        assert_last_event::<T>(Event::ProjectFundsWithdrawn(bob, 0, (10_000u32 * milestone_keys.len() as u32).into(), CurrencyId::Native).into());
    }

    raise_vote_of_no_confidence { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 100_000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 100_000);
        let contribution_amount = 10_000u32;
        let milestone_keys: BoundedMilestoneKeys = vec![0].try_into().unwrap();
        // Setup state: Approved project.
        create_project_common::<T>(contribution_amount.into());
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), vec![0u32].try_into().unwrap(), RoundType::ContributionRound)?;
        run_to_block::<T>(5u32.into());
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), Some(1), 0, contribution_amount.into())?;
        Proposals::<T>::approve(RawOrigin::Root.into(), Some(1), 0, Some(milestone_keys))?;
        
        // (Initiator, ProjectKey)
    }: _(RawOrigin::Signed(alice.clone()) , 0)
    verify {
        assert_last_event::<T>(Event::NoConfidenceRoundCreated(2, 0).into());
    }

    vote_on_no_confidence_round { 
        let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 100_000);
        let charlie: T::AccountId = create_funded_user::<T>("contributor2", 1, 100_000);
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 100_000);
        let contribution_amount = 10_000u32;
        let milestone_keys: BoundedMilestoneKeys = vec![0].try_into().unwrap();
        // Setup state: Approved project.
        create_project_common::<T>(contribution_amount.into());
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), vec![0u32].try_into().unwrap(), RoundType::ContributionRound)?;
        run_to_block::<T>(5u32.into());
        Proposals::<T>::contribute(RawOrigin::Signed(charlie.clone()).into(), Some(1), 0, contribution_amount.into())?;
        Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), Some(1), 0, contribution_amount.into())?;
        Proposals::<T>::approve(RawOrigin::Root.into(), Some(1), 0, Some(milestone_keys))?;
        Proposals::<T>::raise_vote_of_no_confidence(RawOrigin::Signed(alice.clone()).into() , 0)?;
        
        // (Initiator, RoundKey, ProjectKey, boolean)
    }: _(RawOrigin::Signed(charlie), Some(2u32), 0u32, true)
    verify {
        assert_last_event::<T>(Event::NoConfidenceRoundVotedUpon(2, 0).into());
    }


    // Uses refund under hood so we need to account for maximum number of contributors.
    finalise_no_confidence_round { 
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 100_000);
        let contributor: T::AccountId = create_funded_user::<T>("contributor", 0, 100_000);
        let contribution_amount = 10_000u32;
        let milestone_keys: BoundedMilestoneKeys = vec![0].try_into().unwrap();
        let mut contributors: Vec<T::AccountId> = vec![];
        // Setup state: Approved project.
        create_project_common::<T>((contribution_amount * T::MaximumContributorsPerProject::get()).into());
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), vec![0u32].try_into().unwrap(), RoundType::ContributionRound)?;
        run_to_block::<T>(5u32.into());
        
        for i in 0..T::MaximumContributorsPerProject::get() {
            let acc = create_funded_user::<T>("contributor", i, 100_000);
            contributors.push(acc.clone());
            Proposals::<T>::contribute(RawOrigin::Signed(acc.clone()).into(), Some(1), 0, contribution_amount.into())?;
        }
        Proposals::<T>::approve(RawOrigin::Root.into(), Some(1), 0, Some(milestone_keys))?;

        Proposals::<T>::raise_vote_of_no_confidence(RawOrigin::Signed(contributor.clone()).into() ,0)?;
        
        for i in 1..T::MaximumContributorsPerProject::get() {
            Proposals::<T>::vote_on_no_confidence_round(RawOrigin::Signed(contributors[i as usize].clone()).into(), Some(2), 0, false)?;
        }
        // (Contributor, RoundKey, ProjectKey)
    }: _(RawOrigin::Signed(contributor), Some(2u32), 0u32)
    verify {
        assert_last_event::<T>(Event::NoConfidenceRoundFinalised(2, 0).into());
    }

    set_storage_variable {
    }: set_max_project_count_per_round(RawOrigin::Root, u32::MAX)
    verify {
        assert_eq!(MaxProjectCountPerRound::<T>::get(), u32::MAX)  
    }

    refund {
        let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 100_000);
        let contribution_amount = 10_000u32;
        let milestone_keys: BoundedMilestoneKeys = vec![0].try_into().unwrap();
        create_project_common::<T>(contribution_amount.into());
        Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), vec![0u32].try_into().unwrap(), RoundType::ContributionRound)?;
        run_to_block::<T>(5u32.into());
        for i in 0..T::MaximumContributorsPerProject::get() {
            let acc = create_funded_user::<T>("contributor", i, 100_000);
            Proposals::<T>::contribute(RawOrigin::Signed(acc.clone()).into(), Some(1), 0, contribution_amount.into())?;
            if i == T::MaximumContributorsPerProject::get() - 1 {
                Proposals::<T>::raise_vote_of_no_confidence(RawOrigin::Signed(acc.clone()).into() , 0)?;
            }
        }
        
        // (Origin, ProjectKey)
    }:_(RawOrigin::Root, 0)
     verify {
        assert_last_event::<T>(Event::ProjectFundsAddedToRefundQueue(0, (contribution_amount * T::MaximumContributorsPerProject::get()).into()).into());
    }

    refund_item_in_queue {
        run_to_block::<T>(5u32.into());
        let mut accounts: Vec<T::AccountId> = vec![];
        for i in 0..2 {
            let acc = create_funded_user::<T>("contributor", i, 100_000);
            accounts.push(acc);
        }
        
    }: {
        //(From, To, Amount, CurrencyID)
        Proposals::<T>::refund_item_in_queue(&accounts[0], &accounts[1], 10_000u32.into(), CurrencyId::Native)
    }
     verify {
        assert!(T::Currency::total_balance(&accounts[1]) - T::Currency::total_balance(&accounts[0]) == 20_000u32.into());
    }

    split_off_refunds {
        let a in 0..100u32;
        run_to_block::<T>(5u32.into());
        let mut accounts: Vec<T::AccountId> = vec![];
        let mut refunds: Refunds<T> = vec![];
        for i in 0..100usize {
            let acc = create_funded_user::<T>("contributor", i as u32, 100_000);

            accounts.push(acc);
            if i > 0 {
                refunds.push((accounts[0].clone(), accounts[i].clone(), 10_000u32.into(), CurrencyId::Native));
            }
        }
        
    }: {
        //(Refunds, SplitOffIndex)
        Proposals::<T>::split_off_refunds(&mut refunds, a.into())
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
fn create_project_common<T: Config>(contribution: u32) -> T::AccountId {
        
    let milestone_max_count = <MaxProposedMilestones as Get<u32>>::get() as usize;
    let bob: T::AccountId = create_funded_user::<T>("initiator", 1, 1000);
        let project_name: BoundedStringField = b"Imbue's Awesome Initiative".to_vec().try_into().unwrap();
        let project_logo: BoundedStringField = b"Imbue Logo".to_vec().try_into().unwrap();
        let project_description: BoundedDescriptionField = b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.".to_vec().try_into().unwrap();
        let website: BoundedWebsiteUrlField = b"https://imbue.network".to_vec().try_into().unwrap();
        let milestones: BoundedProposedMilestones = vec![ProposedMilestone {
            name: project_logo.clone(),
            percentage_to_unlock: 100 / milestone_max_count as u32,
        }; milestone_max_count].try_into().unwrap();

        let required_funds: BalanceOf<T> = contribution.into();
        let currency_id = CurrencyId::Native;
        
        let _start_block: T::BlockNumber = 0u32.into();

        assert_ok!(Proposals::<T>::create_project(RawOrigin::Signed(bob.clone()).into(), project_name.clone(), project_logo, project_description, website, milestones, required_funds, currency_id));
        bob
    }

fn run_to_block<T: Config>(n: T::BlockNumber) {
    while System::<T>::block_number() < n {
        if System::<T>::block_number() > 1u32.into() {
            Proposals::<T>::on_finalize(System::<T>::block_number());
            System::<T>::on_finalize(System::<T>::block_number());
        }
        System::<T>::set_block_number(System::<T>::block_number() + 1u32.into());
        System::<T>::on_initialize(System::<T>::block_number());
        Proposals::<T>::on_initialize(System::<T>::block_number());
    }
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
