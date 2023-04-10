#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use common_types::CurrencyId;
	
	pub type BalanceOf<T> = <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MultiCurrency: RMultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
	}

	#[pallet::error]
	pub enum Error<T> {
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
        
		#[pallet::call_index(0)]
        #[pallet::weight(100_000)]
        pub fn submit_initial_grant(
            origin: OriginFor<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
			// take deposit to prevent spam

            Self::deposit_event(Event::GrantSubmitted(now));

            Ok(().into())
        }

		#[pallet::call_index(1)]
        #[pallet::weight(100_000)]
        pub fn vote_on_grant(
            origin: OriginFor<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
			// take deposit to prevent spam

            Self::deposit_event(Event::GrantSubmitted(now));

            Ok(().into())
        }

		#[pallet::call_index(2)]
        #[pallet::weight(100_000)]
        pub fn commence_work(
            origin: OriginFor<T>,
        ) -> DispatchResultWithPostInfo {

        }

		// RUNTIME API TO GET THE DEPOSIT ADDRESS FOR THE GRANT ON APPROVAL

	}
}
