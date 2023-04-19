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
		BoundedVec,
		BoundedBTreeMap,
	};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use orml_traits::arithmetic::Bounded;
	use common_types::CurrencyId;
	use sp_runtime::traits::AtLeast32BitUnsigned;
	use pallet_proposals::traits::IntoProposal;

	pub(crate) type BalanceOf<T> = <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BoundedPMilestones<T> = BoundedVec<ProposedMilestoneWithInfo, <T as Config>::MaxMilestonesPerGrant>;
	type BoundedApprovers<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxApprovers>;
	
	type BoundedGrantsSubmitted<T> = BoundedVec<<T as Config>::GrantId, ConstU32<500>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// orml reservable multicurrency.
		type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
		/// The grant ID type.
		type GrantId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + codec::FullCodec + MaxEncodedLen;
		/// Maximum amount of milestones per grant.
		type MaxMilestonesPerGrant: Get<u32>;
		/// The maximum approvers for a given grant.
		type MaxApprovers: Get<u32>;

		/// The type that converts into a proposal for milestone submission.
		type IntoProposal: IntoProposal<
			AccountIdOf<Self>,
			BalanceOf<Self>,
			BlockNumberFor<Self>,
			<Self as pallet_timestamp::Config>::Moment,
		>;
		/// The authority allowed to cancel a pending grant.
		type CancellingAuthority: EnsureOrigin<Self::RuntimeOrigin>;
	}

	/// Stores all the Grants waiting for approval, funding and eventual conversion into milestones.
	/// Key 1: GrantId
	/// Value: Grant<T>
	#[pallet::storage]
	pub type PendingGrants<T: Config> = StorageMap<_, Blake2_128, T::GrantId, Grant<T>, OptionQuery>;


	/// Stores all the grants a user has submitted.
	/// Key 1: AccountId
	/// Key 2: GrantId
	/// Value: ()
	#[pallet::storage]
	pub type GrantsSubmittedBy<T: Config> = StorageDoubleMap<_, Blake2_128, AccountIdOf<T>, Blake2_128, T::GrantId, (), ValueQuery>;
	

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		GrantSubmitted{submitter: AccountIdOf<T>, grant_id: T::GrantId},
		GrantEdited{grant_id: T::GrantId},
		GrantCancelled{grant_id: T::GrantId},

	}

	#[pallet::error]
	pub enum Error<T> {
		/// Milestones must sum to 100
		MustSumTo100,
		/// The GrantId specified cannot be found.
		GrantNotFound,
		/// The grant already exists.
		GrantAlreadyExists,
		/// Overflow Error in pallet-grants.
		Overflow,
		/// Only the submitter can edit this grant.
		OnlySubmitterCanEdit, 
		/// Cannot use a cancelled grant.
		GrantCancelled,
	}

	
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			
			Weight::default()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// A grant starts here with nothing agreed upon and 
		/// probably awaiting much back and forth.
		#[pallet::call_index(0)]
        #[pallet::weight(100_000)]
        pub fn submit_initial_grant(
            origin: OriginFor<T>,
			ipfs_hash: [u8; 32],
			proposed_milestones: BoundedPMilestones<T>,
			assigned_approvers: BoundedApprovers<T>,
        ) -> DispatchResultWithPostInfo {
            let submitter = ensure_signed(origin)?;
			let total_percentage = proposed_milestones.iter().fold(0u32, |acc, x| acc.saturating_add(x.percent.into()));
			ensure!(total_percentage == 100, Error::<T>::MustSumTo100);
			
			// TODO: Ensure that the approvers are in a select group??
			// TODO: take deposit to prevent spam? how else can we prevent spam
			// TODO: GENERATE grant_id. properly. or get as param
			let grant_id: T::GrantId = Default::default();
			ensure!(!PendingGrants::<T>::contains_key(grant_id), Error::<T>::GrantAlreadyExists);

			let grant = Grant {
				milestones: proposed_milestones,
				submitter: submitter.clone(),
				approvers: assigned_approvers,
				ipfs_hash,
				created_on: frame_system::Pallet::<T>::block_number(),
				is_cancelled: false,
			};


			PendingGrants::<T>::insert(&grant_id, grant);
			GrantsSubmittedBy::<T>::insert(&submitter, &grant_id, ());

            Self::deposit_event(Event::<T>::GrantSubmitted{submitter, grant_id});
            Ok(().into())
        }

		/// Edit a grant that has been submitted. 
		/// Fields passed in with None will be ignored and not updated.
		#[pallet::call_index(1)]
        #[pallet::weight(100_000)]
        pub fn edit_grant(
            origin: OriginFor<T>,
			grant_id: T::GrantId,
			edited_milestones: Option<BoundedPMilestones<T>>,
			edited_approvers: Option<BoundedApprovers<T>>,
			edited_ipfs: Option<[u8; 32]>,
        ) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut grant = PendingGrants::<T>::get(grant_id).ok_or(Error::<T>::GrantNotFound)?;

			ensure!(!grant.is_cancelled, Error::<T>::GrantCancelled);
			ensure!(&grant.submitter == &who, Error::<T>::OnlySubmitterCanEdit);

			if let Some(milestones) = edited_milestones {
				grant.milestones = milestones;
			}
			if let Some(approvers) = edited_approvers {
				grant.approvers = approvers;
			}
			if let Some(ipfs) = edited_ipfs {
				grant.ipfs_hash = ipfs;
			}

			PendingGrants::<T>::insert(&grant_id, grant);
            Self::deposit_event(Event::<T>::GrantEdited{grant_id});

			Ok(().into())
        }

		/// Set the grant as cancelled
		#[pallet::call_index(2)]
        #[pallet::weight(100_000)]
        pub fn cancel_grant(
            origin: OriginFor<T>,
			grant_id: T::GrantId,
			as_authority: bool,
        ) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			let mut grant = PendingGrants::<T>::get(&grant_id).ok_or(Error::<T>::GrantNotFound)?;
			if as_authority {
				<T as Config>::CancellingAuthority::ensure_origin(origin)?;
			} else {
				ensure!(grant.submitter == who, Error::<T>::OnlySubmitterCanEdit);
			}

			grant.is_cancelled = true;
			PendingGrants::<T>::insert(&grant_id, grant);
            Self::deposit_event(Event::<T>::GrantCancelled{grant_id});
			
			Ok(().into())
        }

		/// Once you are completely happy with the grant details and are ready to submit to treasury
		/// You call this and itll allow you to generate a project account id.
		#[pallet::call_index(3)]
        #[pallet::weight(100_000)]
        pub fn convert_to_milestones(
            origin: OriginFor<T>,
        ) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// Ensure grant is not cancelled
			Ok(().into())
        }

		// TODO: runtime api to get the deposit address of the grant sovereign account.
	}

	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Grant<T: Config> {
		milestones: BoundedPMilestones<T>,
		submitter: AccountIdOf<T>,
		approvers: BoundedApprovers<T>,
		ipfs_hash: [u8; 32],
		created_on: BlockNumberFor<T>,
		is_cancelled: bool,
	}
	
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub struct ProposedMilestoneWithInfo {
		percent: u8,
		ipfs_hash: [u8; 32],
	}

}
