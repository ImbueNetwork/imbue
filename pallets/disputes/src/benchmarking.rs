#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as PalletDisputes;
use crate::{traits::DisputeRaiser};
use frame_benchmarking::{v2::*};
use frame_support::{assert_ok, BoundedVec};
use sp_runtime::SaturatedConversion;
use orml_traits::MultiCurrency;
use common_types::CurrencyId;


#[benchmarks( where <T as frame_system::Config>::AccountId: AsRef<[u8]>, crate::Event::<T>: Into<<T as frame_system::Config>::RuntimeEvent>)]
mod benchmarks {
    use super::*;
    #[benchmark]
    fn raise_a_dispute() {
        let alice: T::AccountId = create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);
        let jury = get_jury::<T>(vec![alice, bob]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);
        #[block] 
        {
            <Pallet<T> as DisputeRaiser<<T as frame_system::Config>::AccountId>>::raise_dispute(
                10u32.into(),
                alice,
                jury,
                specifics,
            );
        }

        assert!(PalletDisputes::disputes(10u32.into()).is_some());

    }

    impl_benchmark_test_suite!(PalletDisputes, crate::mock::new_test_ext(), crate::mock::Test);
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

pub fn get_jury<T: Config>(
    accounts: Vec<AccountIdOf<T>>,
) -> BoundedVec<AccountIdOf<T>, <T as Config>::MaxJurySize> {
    accounts.try_into().expect("too many jury members")
}

pub fn get_specifics<T: Config>(
    specifics: Vec<T::SpecificId>,
) -> BoundedVec<T::SpecificId, T::MaxSpecifics> {
    specifics.try_into().expect("too many specific ids.")
}

