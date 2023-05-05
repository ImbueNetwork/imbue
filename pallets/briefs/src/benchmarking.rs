#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate as briefs;
use crate::mock::gen_hash;
use crate::Pallet as Briefs;
use crate::{BoundedBriefOwners, BoundedProposedMilestones};
use common_types::CurrencyId;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, traits::Get};
use frame_system::{EventRecord, RawOrigin};
use orml_traits::MultiCurrency;
use pallet_proposals::ProposedMilestone;
use sp_std::str;
use std::convert::TryInto;

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

    create_brief {
        let caller: T::AccountId = create_account_id::<T>("initiator", 1);
        let max_brief_owners: u32 = <T as Config>::MaxBriefOwners::get();
        let brief_owners: BoundedBriefOwners<T> = get_brief_owners::<T>(10).try_into().unwrap();
        let applicant = create_account_id::<T>("applicant", 1);
        let budget = 10_000u32.into();
        let max_milestones: u32 = <T as Config>::MaxMilestonesPerBrief::get();
        let initial_contribution = 5_000u32.into();
        let brief_id = gen_hash(1);
        let milestones = get_milestones::<T>(max_milestones);
        // (origin, brief_owners, applicant, budget, initial_contribution, brief_id, currency_id, milestones)
    }: _(RawOrigin::Signed(caller.clone()), brief_owners, applicant, budget, initial_contribution, brief_id.clone(), CurrencyId::Native, milestones)
    verify {
        assert_last_event::<T>(Event::<T>::BriefSubmitted(caller, brief_id).into());
    }

    contribute_to_brief {
        let caller: T::AccountId = create_account_id::<T>("initiator", 1);
        let brief_owner = create_account_id::<T>("brief_owner", 1);
        let applicant: T::AccountId = create_account_id::<T>("applicant", 1);
        let budget = 10_000u32.into();
        let initial_contribution = 5_000u32.into();
        let contribution = 1_000u32.into();
        let brief_id = gen_hash(1);
        let milestones = get_milestones::<T>(10);
        assert_ok!(Briefs::<T>::create_brief(
            RawOrigin::Signed(caller.clone()).into(),
            vec![brief_owner.clone()].try_into().unwrap(),
            applicant,
            budget,
            initial_contribution,
            brief_id.clone(),
            CurrencyId::Native,
            milestones
        ));
        // (brief_owner, brief_id, contribution)
    }: _(RawOrigin::Signed(brief_owner.clone()), brief_id.clone(), contribution)
    verify {
        assert_last_event::<T>(Event::<T>::BriefContribution(brief_owner, brief_id).into());
    }

    commence_work {
        let caller: T::AccountId = create_account_id::<T>("initiator", 1);
        let brief_owner = create_account_id::<T>("brief_owner", 1);
        let applicant: T::AccountId = create_account_id::<T>("applicant", 1);
        let budget = 10_000u32.into();
        let initial_contribution = 5_000u32.into();
        let brief_id = gen_hash(1);
        let milestones = get_milestones::<T>(10);
        assert_ok!(Briefs::<T>::create_brief(
            RawOrigin::Signed(caller.clone()).into(),
            vec![brief_owner.clone()].try_into().unwrap(),
            applicant.clone(),
            budget,
            initial_contribution,
            brief_id.clone(),
            CurrencyId::Native,
            milestones
        ));
        // (origin, brief_id)
    }: _(RawOrigin::Signed(applicant), brief_id.clone())
    verify {
        assert_last_event::<T>(Event::<T>::BriefEvolution(brief_id).into());
    }

}

fn create_account_id<T: Config>(suri: &'static str, n: u32) -> T::AccountId {
    let user = account(suri, n, SEED);
    let _ = <T::RMultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::deposit(
        CurrencyId::Native,
        &user,
        1_000_000u32.into(),
    );
    user
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

pub(crate) fn get_brief_owners<T: Config>(mut n: u32) -> BoundedBriefOwners<T> {
    let max = <T as briefs::Config>::MaxBriefOwners::get();
    if n > max {
        n = max;
    }
    (0..n)
        .map(|i| create_account_id::<T>("account", i))
        .collect::<Vec<T::AccountId>>()
        .try_into()
        .expect("qed")
}

pub(crate) fn get_milestones<T: Config>(mut n: u32) -> BoundedProposedMilestones<T> {
    let max = <T as briefs::Config>::MaxMilestonesPerBrief::get();
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

impl_benchmark_test_suite!(
    Briefs,
    crate::mock::build_test_externality(),
    crate::mock::Test
);
