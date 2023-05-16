//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v1::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    create_crowdfund {
        let caller: T::AccountId = whitelisted_caller();

        let milestones = get_max_milestones::<T>();

        let required_funds: BalanceOf<T> = u32::MAX.into();
        let currency_id = CurrencyId::Native;
        let agg_hash = H256::from([10u8; 32]);
        let crowdfund_key = 0;
        // (Origin, ipfs_hash, ProposedMilestones, RequiredFunds, CurrencyId)
    }: _(RawOrigin::Signed(whitelisted_caller()), agg_hash, milestones, required_funds, CurrencyId::Native)
    verify {
        assert_last_event::<T>(Event::<T>::CrowdFundCreated(caller, agg_hash, crowdfund_key, required_funds, CurrencyId::Native).into());
    }

    update_crowdfund {
        let milestones = get_max_milestones::<T>();

        let caller = create_crowdfund_common::<T>(u32::MAX.into());

        let required_funds: BalanceOf<T> = u32::MAX.into();
        let currency_id = CurrencyId::Native;
        let agg_hash = H256::from([2; 32]);

        // origin, crowdfund_key, proposed_milestones, required_funds, currency_id, agreement_hash
    }: _(RawOrigin::Signed(caller.clone()), 0,  milestones, required_funds, currency_id, agg_hash)
    verify {
        assert_last_event::<T>(Event::CrowdFundUpdated(caller, 0).into());
    }

    add_crowdfund_whitelist {
        let caller = create_crowdfund_common::<T>(u32::MAX.into());
        let mut bbt : BoundedWhitelistSpots<T> = BTreeMap::new().try_into().unwrap();

        for i in 0..<MaxWhitelistPerCrowdFund as Get<u32>>::get() {
            bbt.try_insert(whitelisted_caller(), 100u32.into()).unwrap();
        }
        // (Origin, CrowdFundKey, BoundedWhitelistSpots)
    }: _(RawOrigin::Signed(caller), 0, bbt)
    verify {
        assert_last_event::<T>(Event::<T>::WhitelistAdded(0, 1u32.into()).into());
    }

    remove_crowdfund_whitelist {
        let caller = create_crowdfund_common::<T>(u32::MAX.into());
        let mut bbt : BoundedWhitelistSpots<T> = BTreeMap::new().try_into().unwrap();

        for i in 0..<MaxWhitelistPerCrowdFund as Get<u32>>::get() {
            bbt.try_insert(whitelisted_caller(), 100u32.into()).unwrap();
        }
        let _ = Proposals::<T>::add_crowdfund_whitelist(RawOrigin::Signed(caller.clone()).into(), 0, bbt);

        // (Origin, CrowdFundKey)
    }: _(RawOrigin::Signed(caller), 0u32)
    verify {
        assert_last_event::<T>(Event::<T>::WhitelistRemoved(0).into());
    }

    open_contributions {
        create_crowdfund_common::<T>(u32::MAX.into())
        // (Origin, CrowdFundKey)c
    }: _(RawOrigin::Root, 0)
    verify {
        assert_last_event::<T>(Event::<T>::FundingRoundCreated(0).into());
    }

    contribute {
        // Setup state.
        let a in 0 .. <MaxCrowdFundKeysPerRound as Get<u32>>::get() - 1;
        let alice: T::AccountId = create_funded_user::<T>("candidate", 1, 100_000);
        let caller: T::AccountId = whitelisted_caller();
        let mut crowdfund_keys: BoundedCrowdFundKeys = vec![].try_into().unwrap();
        for i in 0..<MaxCrowdFundKeysPerRound as Get<u32>>::get() {
            let _caller = create_crowdfund_common::<T>(u32::MAX.into());
            let _ = crowdfund_keys.try_push(i).unwrap();
        }
        let _ = Proposals::<T>::schedule_round(RawOrigin::Root.into(), 3u32.into(), 10u32.into(), crowdfund_keys, RoundType::ContributionRound);

        // Progress the blocks to allow contribution.
        run_to_block::<T>(5u32.into());

        //(Origin, RoundKey, CrowdFundKey, Contribution)
    }: _(RawOrigin::Signed(alice.clone()), Some(1u32), a.into(), 10_000u32.into())
    verify {
        assert_last_event::<T>(Event::<T>::ContributeSucceeded(alice, a.into(), 10_000u32.into(), CurrencyId::Native, 5u32.into()).into());
    }

    approve_crowdfund_for_milestone_submission {
        // Setup state.
        let a in 0 .. <MaxCrowdFundKeysPerRound as Get<u32>>::get() - 1;
        //create a funded user for contribution
        let contribution = 100_000u32;
        let alice: T::AccountId = create_funded_user::<T>("candidate", 1, contribution);
        let mut crowdfund_keys: BoundedCrowdFundKeys = vec![].try_into().unwrap();
        for i in 0..<MaxCrowdFundKeysPerRound as Get<u32>>::get() {
            let _caller = create_crowdfund_common::<T>(100_000u32.into());
            let _ = crowdfund_keys.try_push(i).unwrap();
        }
        let milestone_keys: BoundedMilestoneKeys<T> = (0.. <<T as Config>::MaxMilestonesPerCrowdFund as Get<u32>>::get()).collect::<Vec<u32>>().try_into().unwrap();
        let _ = Proposals::<T>::schedule_round(RawOrigin::Root.into(), 2u32.into(), 10u32.into(), crowdfund_keys, RoundType::ContributionRound);
        run_to_block::<T>(5u32.into());
        let _ = Proposals::<T>::contribute(RawOrigin::Signed(alice.clone()).into(), Some(1u32), a.into(), contribution.into());

        //(Origin, RoundKey, CrowdFundKey, MilestoneKeys)
    }: _(RawOrigin::Root, Some(1), a.into(), Some(milestone_keys))
    verify {
       assert_last_event::<T>(Event::<T>::CrowdFundApproved(1, a.into()).into());
    }

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
fn create_funded_user<T: Config>(
    string: &'static str,
    n: u32,
    balance_factor: u32,
) -> T::AccountId {
    let user = account(string, n, SEED);
    let balance: BalanceOf<T> = balance_factor.into();
    let _ = <T::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::deposit(
        CurrencyId::Native,
        &user,
        balance,
    );
    user
}

fn get_milestones<T: Config>(mut n: u32) -> BoundedProposedMilestones<T> {
    let max = <T as Config>::MaxMilestonesPerCrowdFund::get();
    if n > max {
        n = max;
    }
    let milestones = (0..n)
        .map(|_| ProposedMilestone {
            percentage_to_unlock: 100 / n,
        })
        .collect::<Vec<ProposedMilestone>>()
        .try_into()
        .expect("qed");

    milestones
}

fn get_max_milestones<T: Config>() -> BoundedProposedMilestones<T> {
    let max_milestones: u32 = <T as Config>::MaxMilestonesPerCrowdFund::get();
    get_milestones::<T>(max_milestones)
}
