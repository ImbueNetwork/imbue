#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::test_utils::gen_hash;
use crate::Pallet as Briefs;
use crate::{BoundedBriefOwners, BoundedProposedMilestones};
use common_types::CurrencyId;
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::Get};
use frame_system::{EventRecord, RawOrigin};
use orml_traits::MultiCurrency;
use pallet_proposals::ProposedMilestone;
use sp_arithmetic::per_things::Percent;
use sp_runtime::SaturatedConversion;
use sp_std::{convert::TryInto, str, vec, vec::Vec};

const SEED: u32 = 0;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_brief() {
        let brief_owners = get_max_brief_owners::<T>();
        let caller: T::AccountId = brief_owners[0].clone();
        let applicant = create_account_id::<T>("applicant", 1);
        let budget = 10_000u32.into();
        let initial_contribution = 5_000u32.into();
        let brief_id = gen_hash(1);
        let milestones = get_max_milestones::<T>();
        // (origin, brief_owners, applicant, budget, initial_contribution, brief_id, currency_id, milestones)

        #[extrinsic_call]
        create_brief(
            RawOrigin::Signed(caller.clone()),
            brief_owners,
            applicant,
            budget,
            initial_contribution,
            brief_id,
            CurrencyId::Native,
            milestones,
        );
        assert_last_event::<T>(Event::<T>::BriefSubmitted(caller, brief_id).into());
    }

    #[benchmark]
    fn contribute_to_brief() {
        let brief_owners = get_max_brief_owners::<T>();
        let caller: T::AccountId = brief_owners[0].clone();
        let applicant: T::AccountId = create_account_id::<T>("applicant", 1);
        let budget = 10_000_000_000_000u128.saturated_into();
        let initial_contribution = 5_000_000_000_000u128.saturated_into();
        let contribution = 5_000_000_000_000u128.saturated_into();
        let brief_id = gen_hash(1);
        let milestones = get_max_milestones::<T>();
        assert_ok!(Briefs::<T>::create_brief(
            RawOrigin::Signed(caller).into(),
            brief_owners.clone(),
            applicant,
            budget,
            initial_contribution,
            brief_id,
            CurrencyId::Native,
            milestones
        ));
        let brief_owner: T::AccountId = brief_owners[0].clone();
        // (brief_owner, brief_id, contribution)
        #[extrinsic_call]
        contribute_to_brief(
            RawOrigin::Signed(brief_owner.clone()),
            brief_id,
            contribution,
        );
        assert_last_event::<T>(Event::<T>::BriefContribution(brief_owner, brief_id).into());
    }

    #[benchmark]
    fn commence_work() {
        let brief_owners = get_max_brief_owners::<T>();
        let caller: T::AccountId = brief_owners[0].clone();
        let applicant: T::AccountId = create_account_id::<T>("applicant", 1);
        let budget = 10_000_000_000_000u128.saturated_into();
        let initial_contribution = 5_000_000_000_000u128.saturated_into();
        let brief_id = gen_hash(1);
        let milestones = get_max_milestones::<T>();
        assert_ok!(Briefs::<T>::create_brief(
            RawOrigin::Signed(caller).into(),
            brief_owners,
            applicant.clone(),
            budget,
            initial_contribution,
            brief_id,
            CurrencyId::Native,
            milestones
        ));
        // (origin, brief_id)
        #[extrinsic_call]
        commence_work(RawOrigin::Signed(applicant), brief_id);
        assert_last_event::<T>(Event::<T>::BriefEvolution(brief_id).into());
    }

    #[benchmark]
    fn cancel_brief() {
        let brief_owners = get_max_brief_owners::<T>();
        let caller: T::AccountId = brief_owners[0].clone();
        let applicant: T::AccountId = create_account_id::<T>("applicant", 1);
        let budget = 10_000_000_000_000u128.saturated_into();
        let initial_contribution = 5_000_000_000_000u128.saturated_into();
        let brief_id = gen_hash(1);
        let milestones = get_max_milestones::<T>();
        assert_ok!(Briefs::<T>::create_brief(
            RawOrigin::Signed(caller.clone()).into(),
            brief_owners,
            applicant,
            budget,
            initial_contribution,
            brief_id,
            CurrencyId::Native,
            milestones
        ));
        // (origin, brief_id)
        #[extrinsic_call]
        cancel_brief(RawOrigin::Signed(caller), brief_id);
        assert_last_event::<T>(Event::<T>::BriefCanceled(brief_id).into());
    }

    impl_benchmark_test_suite!(
        Briefs,
        crate::mock::build_test_externality(),
        crate::mock::Test
    );
}

fn create_account_id<T: Config>(suri: &'static str, n: u32) -> T::AccountId {
    let user = account(suri, n, SEED);
    let initial_balance = 1_000_000_000_000_000u128;
    assert_ok!(T::RMultiCurrency::deposit(
        CurrencyId::Native,
        &user,
        initial_balance.saturated_into()
    ));
    user
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

fn get_brief_owners<T: Config>(mut n: u32) -> BoundedBriefOwners<T> {
    let max = <T as Config>::MaxBriefOwners::get();
    if n > max {
        n = max;
    }
    (0..n)
        .map(|i| create_account_id::<T>("brief_owner", i))
        .collect::<Vec<T::AccountId>>()
        .try_into()
        .expect("qed")
}

fn get_max_brief_owners<T: Config>() -> BoundedBriefOwners<T> {
    let max_brief_owners: u32 = <T as Config>::MaxBriefOwners::get();
    get_brief_owners::<T>(max_brief_owners)
}

fn get_milestones<T: Config>(mut n: u32) -> BoundedProposedMilestones<T> {
    let max = <T as Config>::MaxMilestonesPerBrief::get();
    if n > max {
        n = max;
    }

    (0..n)
        .map(|_| ProposedMilestone {
            percentage_to_unlock: Percent::from_percent((100 / n) as u8),
        })
        .collect::<Vec<ProposedMilestone>>()
        .try_into()
        .expect("qed")
}

fn get_max_milestones<T: Config>() -> BoundedProposedMilestones<T> {
    let max_milestones: u32 = <T as Config>::MaxMilestonesPerBrief::get();
    get_milestones::<T>(max_milestones)
}
