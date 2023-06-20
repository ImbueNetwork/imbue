#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

pub mod traits;

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
	use orml_traits::MultiReservableCurrency;

	type AccountIdOf<T> = <T as Config::frame_system>::AccountId;
	type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type EnsureRoot: EnsureOrigin<>
        type MultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
		type DemocracyHandle: traits::DemocracyHandle<AccountIdOf<Self>>;
		type MaxCandidatesPerShortlist: Get<u32>;
	}

	/// Used to map who is a part of the fellowship.
	/// Use contains_key() to find it.
    #[pallet::storage]
    pub type Roles<T> =
        StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Role, OptionQuery>;

	/// Contains the shortlist of candidates to be sent for approval.
	#[pallet::storage]
    pub type CandidateShortlist<T> =
        StorageValue<_, BoundedVec<AccountIdOf<T>, <T as Config>::MaxCandidatesPerShortlist>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		FellowshipAdded(AccountIdOf<T>)
		FellowshipRemoved(AccountIdOf<T>)
	}

	#[pallet::error]
	pub enum Error<T> {

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn force_add_fellowship(origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn force_remove_fellowship(origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn add_candidate_to_shortlist(origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn remove_candidate_from_shortlist(origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}
	}


	impl<T: crate::Config> FellowshipHandler<AccountIdOf<T>> for Pallet<T> {
		fn add_to_fellowship() -> () {
			todo!()
		}

		fn revoke_fellowship() -> () {
			todo!()
		}
		
		fn slash_fellowship_deposit() -> () {
			todo!()
		}
	}
    
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub enum Role {
		Vetter, 
		Fellow
	} 
}


