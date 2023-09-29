#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::traits::DisputeRaiser;
use crate::Pallet as PalletDisputes;
use common_types::CurrencyId;
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, BoundedVec};
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;

#[benchmarks( where <T as frame_system::Config>::AccountId: AsRef<[u8]>, crate::Event::<T>: Into<<T as frame_system::Config>::RuntimeEvent>)]
mod benchmarks {
    use super::*;
    #[benchmark]
    fn raise_a_dispute() {
        let alice: T::AccountId =
            create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
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

    impl_benchmark_test_suite!(
        PalletDisputes,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
