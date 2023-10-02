#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::traits::DisputeRaiser;
use common_types::CurrencyId;
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, BoundedVec};
use orml_traits::MultiCurrency;
use sp_runtime::SaturatedConversion;
use sp_std::vec::Vec;

#[benchmarks( where <T as frame_system::Config>::AccountId: AsRef<[u8]>, crate::Event::<T>: Into<<T as frame_system::Config>::RuntimeEvent>)]
mod benchmarks {
    use super::*;
    #[benchmark]
    fn raise_a_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![alice.clone(), bob]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);
        #[block]
        {
            <crate::Pallet<T> as DisputeRaiser<<T as frame_system::Config>::AccountId>>::raise_dispute(
                10u32.into(),
                alice,
                jury,
                specifics,
            );
        }
    }

    //impl_benchmark_test_suite!(
    //    PalletDisputes,
    //    crate::mock::new_test_ext(),
    //    crate::mock::Test
    //);
}

pub fn get_jury<T: Config>(
    accounts: Vec<<T as frame_system::Config>::AccountId>,
) -> BoundedVec<AccountIdOf<T>, <T as Config>::MaxJurySize> {
    accounts.try_into().expect("too many jury members")
}

pub fn get_specifics<T: Config>(
    specifics: Vec<T::SpecificId>,
) -> BoundedVec<T::SpecificId, T::MaxSpecifics> {
    specifics.try_into().expect("too many specific ids.")
}
