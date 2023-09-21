#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as PalletDisputes;
use crate::{traits::DisputeRaiser};
use frame_benchmarking::{v2::*};
use frame_support::assert_ok;
use sp_runtime::SaturatedConversion;
use orml_traits::{MultiCurrency};
use common_types::CurrencyId;


#[benchmarks( where <T as frame_system::Config>::AccountId: AsRef<[u8]>, crate::Event::<T>: Into<<T as frame_system::Config>::RuntimeEvent>)]
mod benchmarks {
    use super::*;
    #[benchmark]
    fn raise_a_dispute() {
        let alice: T::AccountId = create_funded_user::<T>("alice", 1, 1_000_000_000_000_000_000u128);
        let bob: T::AccountId = create_funded_user::<T>("bob", 1, 1_000_000_000_000_000_000u128);
        let charlie: T::AccountId = create_funded_user::<T>("charlie", 1, 1_000_000_000_000_000_000u128);
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![alice, bob]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        #[block] 
        {
            <PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
                dispute_key,
                alice,
                jury,
                specifics,
            );
        }
        
        assert!(PalletDisputes::disputes(dispute_key).is_some());
        assert_eq!(1, PalletDisputes::disputes(dispute_key).iter().count());
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

