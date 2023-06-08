#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::test_utils::gen_grant_id;
use crate::Pallet as Grants;
use crate::{BoundedApprovers, BoundedPMilestones};
use common_types::{CurrencyId, TreasuryOrigin};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, traits::Get};
use frame_system::{EventRecord, RawOrigin};
use orml_traits::GetByKey;
use orml_traits::{MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
use pallet_proposals::ProposedMilestone;
use sp_arithmetic::per_things::Percent;
use sp_core::sr25519::{Public, Signature};
use sp_runtime::SaturatedConversion;
use sp_std::{convert::TryInto, str, vec::Vec};

const SEED: u32 = 0;

benchmarks! {
    where_clause {
        where T::AccountId: AsRef<[u8]>,
    }

    submit_initial_grant {
        let submitter: T::AccountId = create_account_id::<T>("submitter", 1);
        let proposed_milestones = get_max_milestones::<T>();
        let assigned_approvers = get_max_approvers::<T>();
        let amount = 1_000_000u32.into();
        let grant_id = gen_grant_id(1);
        // origin, propose_milestones, assigned_approvers, currency_id, amount_requested, treasury_origin, grant_id
    }: _(RawOrigin::Signed(submitter.clone()), proposed_milestones, assigned_approvers, CurrencyId::Native, amount, TreasuryOrigin::Kusama, grant_id)
    verify {
        assert_last_event::<T>(Event::<T>::GrantSubmitted{submitter, grant_id}.into());
    }

    edit_grant {
        let submitter: T::AccountId = create_account_id::<T>("submitter", 1);
        let proposed_milestones = get_max_milestones::<T>();
        let assigned_approvers = get_max_approvers::<T>();
        let grant_id = gen_grant_id(1);
        let amount_requested = 1_000_000u32.into();

        assert_ok!(Grants::<T>::submit_initial_grant(
            RawOrigin::Signed(submitter.clone()).into(),
            proposed_milestones.clone(),
            assigned_approvers.clone(),
            CurrencyId::Native,
            amount_requested,
            TreasuryOrigin::Kusama,
            grant_id.clone()
        ));

        // origin, grant_id, edited_milestones, edited_approvers, edited_currency_id, edited_amount_requested, edited_treasury_origin
    }: _(RawOrigin::Signed(submitter), grant_id.clone(), Some(proposed_milestones), Some(assigned_approvers), Some(CurrencyId::Native), Some(amount_requested), Some(TreasuryOrigin::Kusama))
    verify {
        assert_last_event::<T>(Event::<T>::GrantEdited{grant_id}.into());
    }

    cancel_grant {
        let submitter: T::AccountId = create_account_id::<T>("submitter", 1);
        let proposed_milestones = get_max_milestones::<T>();
        let assigned_approvers = get_max_approvers::<T>();
        let grant_id = gen_grant_id(1);
        let amount_requested = 1_000_000u32.into();

        assert_ok!(Grants::<T>::submit_initial_grant(
            RawOrigin::Signed(submitter.clone()).into(),
            proposed_milestones.clone(),
            assigned_approvers.clone(),
            CurrencyId::Native,
            amount_requested,
            TreasuryOrigin::Kusama,
            grant_id.clone()
        ));

        // origin, grant_id, as_authority
    }: _(RawOrigin::Root, grant_id.clone(), true)
    verify {

    }

    convert_to_project {
        let submitter: T::AccountId = create_account_id::<T>("submitter", 1);
        let proposed_milestones = get_max_milestones::<T>();
        let assigned_approvers = get_max_approvers::<T>();
        let grant_id = gen_grant_id(1);
        let amount_requested = 1_000_000u32.into();

        assert_ok!(Grants::<T>::submit_initial_grant(
            RawOrigin::Signed(submitter.clone()).into(),
            proposed_milestones.clone(),
            assigned_approvers.clone(),
            CurrencyId::Native,
            amount_requested,
            TreasuryOrigin::Kusama,
            grant_id.clone()
        ));

        // origin, grant_id
    }: _(RawOrigin::Signed(submitter.clone()), grant_id.clone())
    verify {
        let grant = PendingGrants::<T>::get(grant_id).unwrap();
        assert!(grant.is_converted);
    }
}

fn get_max_approvers<T: Config>() -> BoundedApprovers<T> {
    let n = <T as Config>::MaxApprovers::get();
    (0..n)
        .map(|i| create_account_id::<T>("brief_owner", i))
        .collect::<Vec<T::AccountId>>()
        .try_into()
        .expect("qed")
}

fn get_max_milestones<T: Config>() -> BoundedPMilestones<T> {
    let n = <T as Config>::MaxMilestonesPerGrant::get();
    let milestones = (0..n)
        .map(|_| ProposedMilestone {
            percentage_to_unlock: Percent::from_percent((100 / n) as u8),
        })
        .collect::<Vec<ProposedMilestone>>()
        .try_into()
        .expect("qed");

    milestones
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

fn create_account_id<T: Config>(suri: &'static str, n: u32) -> T::AccountId {
    let user = account(suri, n, SEED);
    let initial_balance: _ = 10_000_000_000_000_000u128;
    assert_ok!(T::RMultiCurrency::deposit(
        CurrencyId::Native,
        &user,
        initial_balance.saturated_into()
    ));
    user
}
impl_benchmark_test_suite!(Grants, crate::mock::new_test_ext(), crate::mock::Test);
