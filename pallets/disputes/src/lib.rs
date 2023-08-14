#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub mod traits;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type DisputeKey = u32;
	pub(crate) type ProjectKey = u32;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
    pub type Disputes<T> =
        StorageMap<_, Blake2_128Concat, ProjectKey, AccountIdOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DisputeRaised {who: AccountIdOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn handle_dispute(
			origin: OriginFor<T>,
            who: AccountIdOf<T>,
			projectKey: ProjectKey) -> DispatchResult {

			let origin = ensure_signed(origin)?;
            Disputes::<T>::insert(projectKey,who.clone());


			Self::deposit_event(Event::<T>::DisputeRaised {who: who.clone()});
			Ok(())
		}

	}

	#[derive(Encode, Decode, PartialEq, Eq, Copy, Clone, Debug, MaxEncodedLen)]
    pub enum Jury {
		Fellowship,
		Contributors,
		Canonical,
    }

}