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

	use frame_support::{
		pallet_prelude::*,
		traits::{Get},
		BoundedBTreeMap
	};
	use frame_system::{
		pallet_prelude::*,
	};
	use common_types::CurrencyId;
	use orml_traits::{
		MultiReservableCurrency,
		MultiCurrency,
	};
	use sp_core::{Hasher, H256};


	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
	type BoundedApplications<T> = BoundedBTreeMap<AccountIdOf<T>, (), <T as Config>::MaximumApplicants>;
	
	type BriefHash = H256;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + proposals::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
		/// The minimum deposit required to submit a brief
		// SHOULD THIS BE AS A PERCENT OF BOUNTY? TODO:.
		type MinimumDeposit: Get<BalanceOf<Self>>;
		/// The minimum bounty required to submit a brief.
		type MinimumBounty: Get<BalanceOf<Self>>;
		/// Maximum amount of applicants to a brief.
		type MaximumApplicants: Get<u32>;
		// The fee taken for submitting a brief could be a deposit?
		//type BriefSubmissionFee: Get<Percent>;
		/// Hasher used to generate brief hash
		type BriefHasher: Hasher;
	}

	#[pallet::storage]
	#[pallet::getter(fn briefs)]
	pub type Briefs<T> = CountedStorageMap<_, Blake2_128Concat, BriefHash, BriefData<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn brief_applications)]
	pub type BriefApplications<T> = StorageMap<_, Blake2_128Concat, BriefHash, BoundedApplications<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn approved_accounts)]
	pub type ApprovedAccounts<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, (), OptionQuery>;
	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		BriefSubmitted(BriefHash),
		ApplicationSubmitted(AccountIdOf<T>)
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The deposit you have sent is below the minimum requirement.
		DepositBelowMinimum,
		/// The bounty you have set is below the minimum requirement.
		BountyBelowMinimum,
		/// The contribution you have sent is more than the bounty total.
		ContributionMoreThanBounty,
		/// Only approved account can apply for briefs.
		OnlyApprovedAccountPermitted,
		/// You have already applied for this brief.
		AlreadyApplied,
		/// Brief already exists in the block, please don't submit duplicates.
		BriefAlreadyExists,
		/// Maximum Applications have been reached.
		MaximumApplicants,
		/// Brief not found.
		BriefNotFound,
		/// The BriefId generation failed.
		BriefHashingFailed,

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Submit a brief to recieve applications.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn submit_brief(origin: OriginFor<T>, off_chain_ref_id: u32, bounty_total: BalanceOf<T>, initial_contribution: BalanceOf<T>, currency_id: CurrencyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(initial_contribution >= <T as Config>::MinimumDeposit::get(), Error::<T>::DepositBelowMinimum);
			ensure!(bounty_total >= <T as Config>::MinimumBounty::get(), Error::<T>::BountyBelowMinimum);
			ensure!(bounty_total >= initial_contribution, Error::<T>::ContributionMoreThanBounty);

			// This will prevent duplicates.
			// Malicious users can still submit briefs without an off chain storage id (or an invalid one).
			// Therefore we must check that this item does exist in storage in an ocw and possible slash those who are malicious.
			// append_to_id_verification(&off_chain_ref_id);
			// Update db from ocw to include the new brief_id??.

			let brief_id: BriefHash = BriefPreImage::<T>::generate_hash(&who, &bounty_total, &currency_id, off_chain_ref_id)?;
			ensure!(Briefs::<T>::get(brief_id).is_none(), Error::<T>::BriefAlreadyExists);

			let new_brief = BriefData {
				created_by: who.clone(),
				bounty_total,
				currency_id,
				off_chain_ref_id,
				current_contribution: initial_contribution,
				created_at: frame_system::Pallet::<T>::block_number(),
			};

			<T as Config>::RMultiCurrency::reserve(currency_id, &who, initial_contribution)?;
			Briefs::<T>::insert(brief_id, new_brief);
			Self::deposit_event(Event::<T>::BriefSubmitted(brief_id));

			Ok(())
		}


		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn submit_application(origin: OriginFor<T>, brief_id: BriefHash) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let is_approved = ApprovedAccounts::<T>::get(&who).is_some();
			ensure!(is_approved, Error::<T>::OnlyApprovedAccountPermitted);

			if let Some(mut applicants) = BriefApplications::<T>::get(brief_id) {
				ensure!(applicants.get(&who).is_none(), Error::<T>::AlreadyApplied);
				if applicants.try_insert(who.clone(), ()).is_ok() {
					BriefApplications::<T>::insert(brief_id, applicants);
				} else {
					return Err(Error::<T>::MaximumApplicants.into())
				};
			} else {
				return Err(Error::<T>::BriefNotFound.into())
			}; 

			Self::deposit_event(Event::<T>::ApplicationSubmitted(who));
			Ok(())
		}
	}


	#[derive(Encode, Hash)]
	pub struct BriefPreImage<T: Config> {
		created_by: Vec<u8>,
		bounty_total: Vec<u8>,
		currency_id: Vec<u8>,
		off_chain_ref_id: u32,
		phantom: PhantomData<T>,
	}

	impl <T: Config> BriefPreImage<T> {
		fn new<'a>(created_by: &'a AccountIdOf<T>, bounty_total: &'a BalanceOf<T>, currency_id: &'a CurrencyId, off_chain_ref_id: u32) -> Self {
			Self {
				created_by: <AccountIdOf<T> as Encode>::encode(created_by),
				bounty_total: <BalanceOf<T> as Encode>::encode(bounty_total),
				currency_id: <CurrencyId as Encode>::encode(currency_id),
				off_chain_ref_id,
				phantom: PhantomData
			}
		}
		
		pub fn generate_hash<'a>(created_by: &'a AccountIdOf<T>, bounty_total: &'a BalanceOf<T>, currency_id: &'a CurrencyId, off_chain_ref_id: u32) -> Result<BriefHash, DispatchError> {
			let preimage: Self = Self::new(created_by, bounty_total, currency_id, off_chain_ref_id); 
			let encoded = <BriefPreImage<T> as Encode>::encode(&preimage);
			let maybe_h256: Result<[u8; 32], _> = <<T as Config>::BriefHasher as Hasher>::hash(&encoded).as_ref().try_into(); 
			if let Ok(h256) = maybe_h256 {
				Ok(H256::from_slice(h256.as_slice()))
			} else {
				Err(Error::<T>::BriefHashingFailed.into())
			}
		}
	}

	

	/// The data assocaited with a Brief, 
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub struct BriefData<AccountId, Balance, BlockNumber> {
		// looking to store minimal data on chain. 
		// We can get the rest of the data from the backend dapp.
		created_by: AccountId,
		bounty_total: Balance,
		currency_id: CurrencyId,
		current_contribution: Balance,
		off_chain_ref_id: u32,
		created_at: BlockNumber,

		//milestones?
	}



}
