#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type BriefId = [u8; 32];
type AccountIdOf<T> = <T as frame_system::Config>::AccountId
type BalanceOf<T> = <<T as Config>::RMultiCurrency as ReservableMultiCurrency<AccountIdOf<T>>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Get}
	};
	use frame_system::pallet_prelude::*;
	use common_types::CurrencyId;
	use orml_traits::ReservableMultiCurrency;
	use pallet_proposals::{Config};
	

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + proposals::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type RMultiCurrency: ReservableMultiCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
		/// The minimum deposit required to submit a brief.
		type MinimumDeposit: Get<BalanceOf<T>>;
		/// The minimum bounty required to submit a brief.
		type MinimumBounty: Get<BalanceOf<T>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Briefs<T> = StorageMap<_, BriefId, BriefInfo>;

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type BriefApplications<T> = StorageMap<_, BriefId, Vec<Application<T>>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		BriefSubmitted,
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The deposit you have sent is below the minimum requirement.
		DepositBelowMinimum,
		/// The bounty you have set is below the minimum requirement.
		BountyBelowMinimum,

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn submit_brief(origin: OriginFor<T>, brief_id: BriefId, bounty_total: BalanceOf<T>, initial_contribution: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(initial_contribution >= <T as Config>::MinimumDeposit::get(), Error::<T>::DepositBelowMinimum);
			ensure!(bounty_total >= <T as Config>::MinimumBounty::get(), Error::<T>::BountyBelowMinimum);

			ensure!(brief_id)
		}

	}

	/// An application to a brief, used to decide who will do the work.
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
	struct Application<T: frame_system::Config> {
		who: AccountIdOf<T>,
		// do we need info on chain? arguably only the account id for the who clause.
		//db_id: u64, 		
	}	

	/// The data assocaited with a Brief, 
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
	struct BriefData<T: Config> {
		// looking to store minimal data on chain. 
		// We can get the rest of the data from the backend dapp.
		created_by: AccountIdOf<T>
		brief_id: BriefId,
		bounty_total: BalanceOf<T>,
		currency: CurrencyId,
		current_contribution: BalanceOf<T>,
		//milestones?

	}
}
