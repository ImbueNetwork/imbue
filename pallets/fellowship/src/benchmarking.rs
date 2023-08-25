#![cfg(feature = "runtime-benchmarks")]
use super::*;
use crate::Pallet as Fellowship;
use crate::{traits::FellowshipHandle, Role};
use common_types::CurrencyId;
use frame_benchmarking::{v2::*};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::SaturatedConversion;
use frame_system::Pallet as System;

#[benchmarks( where
    <T as frame_system::Config>::AccountId: AsRef<[u8]>,
    crate::Event::<T>: Into<<T as frame_system::Config>::RuntimeEvent>,
)]

mod benchmarks {
    use super::*;

    #[benchmark]
    fn add_to_fellowship() {
        let alice: T::AccountId = create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);

        #[block]
        {
            <crate::Pallet<T> as FellowshipHandle<<T as frame_system::Config>::AccountId>>::add_to_fellowship(&alice, Role::Vetter, 10, Some(&bob), true);
        }
        System::<T>::assert_last_event(Event::<T>::FellowshipAdded{who: alice.clone(), role: Role::Vetter}.into());
    }

    #[benchmark]
    fn force_add_fellowship() {
        let alice: T::AccountId = create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        #[extrinsic_call]
        force_add_fellowship(RawOrigin::Root, alice, Role::Freelancer, 10);
        System::<T>::assert_last_event(Event::<T>::FellowshipAdded{who: alice.clone(), role: Role::Vetter}.into());
    }

    #[benchmark]
    fn leave_fellowship() {
        let alice: T::AccountId = create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);
        <crate::Pallet<T> as FellowshipHandle<<T as frame_system::Config>::AccountId>>::add_to_fellowship(&alice, Role::Vetter, 10, Some(&bob), true);

        #[extrinsic_call]
        leave_fellowship(&alice);

        System::<T>::assert_last_event(Event::<T>::FellowshipRemoved{who: alice.clone()}.into());
    }

    #[benchmark]
    fn force_remove_and_slash_fellowship() {
        let alice: T::AccountId = create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);
        <crate::Pallet<T> as FellowshipHandle<<T as frame_system::Config>::AccountId>>::add_to_fellowship(&alice, Role::Vetter, 10, Some(&bob), true);

        #[extrinsic_call]
        force_remove_and_slash_fellowship(RawOrigin::Root, alice.clone());
        System::<T>::assert_last_event(Event::<T>::FellowshipRemoved{who: alice.clone()}.into());
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
        <T as frame_system::Config>::AccountId,
    >>::deposit(
        CurrencyId::Native, &user, balance_factor.saturated_into()
    ));
    user
}
