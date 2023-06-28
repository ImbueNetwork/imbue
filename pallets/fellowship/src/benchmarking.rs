use super::*;
#[allow(unused)]
use crate::Pallet as Fellowship;
use frame_benchmarking::v1::{benchmarks, whitelisted_caller, account};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use common_types::CurrencyId;
use crate::{Role, traits::FellowshipHandle};
use sp_runtime::SaturatedConversion;
use orml_traits::{MultiCurrency, MultiReservableCurrency};

benchmarks! {
  add_to_fellowship {
      let alice: T::AccountId = create_funded_user::<T>("contributor", 1, 1_000_000_000_000_000_000u128);
  }: {
    <crate::Pallet<T> as FellowshipHandle<<T as frame_system::Config>::AccountId>>::add_to_fellowship(&alice, Role::Vetter);
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