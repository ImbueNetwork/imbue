#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Fellowship;
use crate::{traits::FellowshipHandle, Config, Role};
use common_types::CurrencyId;
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::Pallet as System;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use sp_std::vec;

#[benchmarks( where BlockNumberFor<T>: AsRef<[u8]>, crate::Event::<T>: Into<<T as frame_system::Config>::RuntimeEvent>)]
mod benchmarks {
    use super::*;
    #[benchmark]
    fn add_to_fellowship() {
        let alice: T::AccountId =
            create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);

        #[block]
        {
            <crate::Pallet<T> as FellowshipHandle<BlockNumberFor<T>>>::add_to_fellowship(&alice, Role::Vetter, 10, Some(&bob), true);
        }
    }

    #[benchmark]
    fn force_add_fellowship() {
        let alice: T::AccountId =
            create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        #[extrinsic_call]
        force_add_fellowship(RawOrigin::Root, alice.clone(), Role::Freelancer, 10);
        System::<T>::assert_last_event(
            Event::<T>::FellowshipAdded {
                who: alice,
                role: Role::Freelancer,
            }
            .into(),
        );
    }

    #[benchmark]
    fn leave_fellowship() {
        let alice: T::AccountId =
            create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);
        <crate::Pallet<T> as FellowshipHandle<BlockNumberFor<T>>>::add_to_fellowship(&alice, Role::Vetter, 10, Some(&bob), true);

        #[extrinsic_call]
        leave_fellowship(RawOrigin::Signed(alice.clone()));

        System::<T>::assert_last_event(Event::<T>::FellowshipRemoved { who: alice }.into());
    }

    #[benchmark]
    fn force_remove_and_slash_fellowship() {
        let alice: T::AccountId =
            create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);
        <crate::Pallet<T> as FellowshipHandle<BlockNumberFor<T>>>::add_to_fellowship(&alice, Role::Vetter, 10, Some(&bob), true);

        #[extrinsic_call]
        force_remove_and_slash_fellowship(RawOrigin::Root, alice.clone());
        System::<T>::assert_last_event(Event::<T>::FellowshipSlashed { who: alice }.into());
    }

    #[benchmark]
    fn add_candidate_to_shortlist() {
        let alice: T::AccountId =
            create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);
        <crate::Pallet<T> as FellowshipHandle<BlockNumberFor<T>>>::add_to_fellowship(&alice, Role::Vetter, 10, Some(&bob), true);

        #[extrinsic_call]
        add_candidate_to_shortlist(RawOrigin::Signed(alice), bob.clone(), Role::Vetter, 10);
        System::<T>::assert_last_event(Event::<T>::CandidateAddedToShortlist { who: bob }.into());
    }

    #[benchmark]
    fn remove_candidate_from_shortlist() {
        let alice: T::AccountId =
            create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);
        <crate::Pallet<T> as FellowshipHandle<BlockNumberFor<T>>>::add_to_fellowship(&alice, Role::Vetter, 10, Some(&bob), true);
        assert_ok!(Fellowship::<T>::add_candidate_to_shortlist(
            RawOrigin::Signed(alice.clone()).into(),
            bob.clone(),
            Role::Vetter,
            10,
        ));

        #[extrinsic_call]
        remove_candidate_from_shortlist(RawOrigin::Signed(alice), bob.clone());
        System::<T>::assert_last_event(
            Event::<T>::CandidateRemovedFromShortlist { who: bob }.into(),
        );
    }

    #[benchmark]
    fn pay_deposit_to_remove_pending_status() {
        let bob: T::AccountId = account("bob", 1, 0);
        let charlie: T::AccountId =
            create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);

        <crate::Pallet<T> as FellowshipHandle<BlockNumberFor<T>>>::add_to_fellowship(&bob, Role::Vetter, 10, Some(&charlie), true);
        assert_ok!(<T::MultiCurrency as MultiCurrency<
            BlockNumberFor<T>,
        >>::deposit(
            CurrencyId::Native,
            &bob,
            1_000_000_000_000_000_000u128.saturated_into()
        ));

        #[extrinsic_call]
        pay_deposit_to_remove_pending_status(RawOrigin::Signed(bob.clone()));
        System::<T>::assert_last_event(
            Event::<T>::FellowshipAdded {
                who: bob,
                role: Role::Vetter,
            }
            .into(),
        );
    }

    impl_benchmark_test_suite!(Fellowship, crate::mock::new_test_ext(), crate::mock::Test);
}

pub fn create_funded_user<T: Config>(
    seed: &'static str,
    n: u32,
    balance_factor: u128,
) -> T::AccountId {
    let user = account(seed, n, 0);
    assert_ok!(<T::MultiCurrency as MultiCurrency<
        BlockNumberFor<T>,
    >>::deposit(
        CurrencyId::Native, &user, balance_factor.saturated_into()
    ));
    user
}
