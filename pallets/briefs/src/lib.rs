#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Get}
	};
	use frame_system::pallet_prelude::*;
	use common_types::CurrencyId;
	use orml_traits::{MultiReservableCurrency, MultiCurrency};

	
	// A unique hash of the brief in the db.
	pub type BriefId = [u8; 32];

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
	type BoundedApplications<T> = BoundedVec<Application<AccountIdOf<T>>, <T as Config>::MaximumApplicants>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + proposals::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
		/// The minimum deposit required to submit a brief.
		type MinimumDeposit: Get<BalanceOf<Self>>;
		/// The minimum bounty required to submit a brief.
		type MinimumBounty: Get<BalanceOf<Self>>;
		// Maximum amount of applicants to a brief.
		type MaximumApplicants: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn briefs)]
	pub type Briefs<T> = StorageMap<_, Blake2_128Concat, BriefId, BriefData<AccountIdOf<T>, BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn brief_applications)]
	pub type BriefApplications<T> = StorageMap<_, Blake2_128Concat, BriefId, BoundedApplications<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		BriefSubmitted(BriefId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The deposit you have sent is below the minimum requirement.
		DepositBelowMinimum,
		/// The bounty you have set is below the minimum requirement.
		BountyBelowMinimum,
		/// The contribution you have sent is more than the bounty total.
		ContributionMoreThanBounty,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn submit_brief(origin: OriginFor<T>, brief_id: BriefId, bounty_total: BalanceOf<T>, initial_contribution: BalanceOf<T>, currency_id: CurrencyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(initial_contribution >= <T as Config>::MinimumDeposit::get(), Error::<T>::DepositBelowMinimum);
			ensure!(bounty_total >= <T as Config>::MinimumBounty::get(), Error::<T>::BountyBelowMinimum);
			ensure!(bounty_total >= initial_contribution, Error::<T>::ContributionMoreThanBounty);
			// ensure that the brief_id is a legitimate brief (can be done in offchain worker as we need to make a req)!
			// Take fee for brief

			<T as Config>::RMultiCurrency::reserve(currency_id, &who, initial_contribution)?;

			let new_brief = BriefData {
				created_by: who,
				bounty_total,
				current_contribution: initial_contribution,
				currency_id
				// milestones
			};

			Briefs::<T>::insert(brief_id, new_brief);

			Self::deposit_event(Event::<T>::BriefSubmitted(brief_id));

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn submit_application(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Ok(())
		
		}
	}


	/// An application to a brief, used to decide who will do the work.
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub struct Application<AccountId> {
		who: AccountId,
		// do we need info on chain? arguably only the account id for the who clause.
		//db_id: u64, 		
	}	

	/// The data assocaited with a Brief, 
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub struct BriefData<AccountId, Balance> {
		// looking to store minimal data on chain. 
		// We can get the rest of the data from the backend dapp.
		created_by: AccountId,
		bounty_total: Balance,
		currency_id: CurrencyId,
		current_contribution: Balance,
		//milestones?
	}
}
