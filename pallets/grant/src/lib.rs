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

	pub(crate) type BalanceOf<T> = <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BoundedPMilestones<T> = BoundedVec<ProposedMilestoneWithInfo, <T as Config>::MaxMilestonesPerGrant>;
	type BoundedApprovers<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxApprovers>;
	type MaxGrantsExpiringPerBlock = ConstU32<100>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
		/// The grant ID type.
		type GrantId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + codec::FullCodec + MaxEncodedLen;
		type MaxMilestonesPerGrant: Get<u32>;
		/// The maximum approvers for a given grant, should be set to < 100.
		type MaxApprovers: Get<u32>;
	
		// Used to remove ignored grants and keep a clean system.
		type GrantVotingPeriod: Get<<Self as frame_system::Config>::BlockNumber>;
	}

	/// Stores all the Grants waiting for unanimous approval and eventual conversion into milestones.
	/// Key 1: GrantId
	/// Value: Grant<T>
	#[pallet::storage]
	pub type PendingGrants<T: Config> = StorageMap<_, Blake2_128, T::GrantId, Grant<T>, OptionQuery>;

	/// Used to hold all the votes by the approvers on a given grant.
	/// Key 1: GrantId
	/// Key 2: AccountId
	/// Value: VoteType
	#[pallet::storage]
	pub type GrantVotes<T: Config> = StorageDoubleMap<_, Blake2_128, T::GrantId, Blake2_128, AccountIdOf<T>, VoteType, OptionQuery>;

	/// Used to hold to votes count of a grant, an optimisation over iterating keys.
	/// Key 1: GrantId
	/// Value: u32 (Amount of distinct votes on a given grant)
	#[pallet::storage]
	pub type GrantVoteCount<T: Config> = StorageMap<_, Blake2_128, T::GrantId, u32, ValueQuery>;

	/// The grants the expire on a given block.
	/// Used to clean up storage and remove unwanted proposals.
    /// Key 1: BlockNumber
    /// Value: BoundedVec<GrantIds>
	#[pallet::storage]
	pub type GrantVotingExpiration<T: Config> = StorageMap<_, Blake2_128, BlockNumberFor<T>, BoundedVec<T::GrantId, MaxGrantsExpiringPerBlock>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		GrantSubmitted{submitter: AccountIdOf<T>, grant_id: T::GrantId},
		GrantVotedUpon{voter: AccountIdOf<T>, grant_id: T::GrantId, way: VoteType},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Milestones must sum to 100
		MustSumTo100,
		/// The GrantId specified cannot be found.
		GrantNotFound,
		/// Only appointed approvers and vote on a grant submission.
		OnlyApproversCanVote,
		/// Maximum grants per block reached try again next block.
		MaxGrantsPerBlockReached,
		/// Overflow Error in pallet-grants.
		Overflow,
	}

	
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let mut weight = Weight::default();
			let expiring_grants = GrantVotingExpiration::<T>::get(n);
            weight += T::DbWeight::get().reads(2);
			
			// Remove all the grants from storage that have reached expiry 
			let _ = expiring_grants.iter().map(|grant_id| {
            	weight += T::DbWeight::get().reads_writes(1, 1);
				PendingGrants::<T>::remove(grant_id);
			}).collect::<Vec<_>>();

			weight + T::DbWeight::get().reads(1)
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
			// TODO: take deposit to prevent spam? how else can we prevent spam
			let total_percentage = proposed_milestones.iter().fold(0u32, |acc, x| acc.saturating_add(x.percent.into()));
			ensure!(total_percentage == 100, Error::<T>::MustSumTo100);
			
			// TODO: Ensure that the approvers are in a select group??
			//ensure!()

			// TODO: GENERATE grant_id. properly. or get as param
			let grant_id: T::GrantId = Default::default();
			
			let grant = Grant {
				milestones: proposed_milestones,
				submitter: submitter.clone(),
				approvers: assigned_approvers,
				ipfs_hash,
				created_on: frame_system::Pallet::<T>::block_number(),
			};

			let exp_block: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number() + <T as Config>::GrantVotingPeriod::get();
			let _ = GrantVotingExpiration::<T>::try_mutate(exp_block, |grant_ids| {
				let _ = grant_ids.try_push(grant_id).map_err(|_| Error::<T>::MaxGrantsPerBlockReached)?;
				Ok::<(), DispatchError>(())
			})?;
			PendingGrants::<T>::insert(&grant_id, grant);

            Self::deposit_event(Event::<T>::GrantSubmitted{submitter, grant_id});
            Ok(().into())
        }

		/// For the people approving, they must submit the vote register their intention on the grant proposal. 
		/// This can be called multiple times to allow for editing of the vote.
		#[pallet::call_index(1)]
        #[pallet::weight(100_000)]
        pub fn vote_on_grant(
            origin: OriginFor<T>,
			vote: VoteType,
			grant_id: T::GrantId,
        ) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
			let grant: Grant<T> = PendingGrants::<T>::get(grant_id).ok_or(Error::<T>::GrantNotFound)? ;
			let mut is_new_vote = true;

			ensure!(grant.approvers.iter().any(|approver|approver == &voter), Error::<T>::OnlyApproversCanVote);
			
			GrantVotes::<T>::mutate(&grant_id, &voter, |v|{
				if v.is_some() {is_new_vote = false};
				*v = Some(vote.clone()); 
			});

			if is_new_vote {
				let count_of_votes = GrantVoteCount::<T>::take(&grant_id).saturating_add(1);
				GrantVoteCount::<T>::insert(&grant_id, count_of_votes);
				// This shouldnt be too expensive as there is a limit of how many approvers there are (MaxApprovers)
				if count_of_votes as usize == grant.approvers.len() {
					let exp_block = grant.created_on + <T as Config>::GrantVotingPeriod::get();
					GrantVotingExpiration::<T>::try_mutate(exp_block, |grant_ids| {
						*grant_ids = grant_ids
						.iter()
						.cloned()
						.filter(|&id| id != grant_id)
						.collect::<Vec<_>>()
						.try_into()
						.map_err(|_| Error::<T>::Overflow)?;
						
						Ok::<(), DispatchError>(())
					})?;
				}
			}

            Self::deposit_event(Event::<T>::GrantVotedUpon{voter, grant_id, way: vote});
            Ok(().into())
        }

		/// Accept a grant to stop it from auto expiring.
		/// Call this if you want to keep a grant but one or many approvers is not responding.
		#[pallet::call_index(2)]
        #[pallet::weight(100_000)]
        pub fn keep_grant_from_expiring(
            origin: OriginFor<T>,
        ) -> DispatchResultWithPostInfo {

			Ok(().into())
        }

		/// Remove the grant from storage.
		#[pallet::call_index(3)]
        #[pallet::weight(100_000)]
        pub fn cancel_grant(
            origin: OriginFor<T>,
        ) -> DispatchResultWithPostInfo {
			
			Ok(().into())
        }

		#[pallet::call_index(4)]
        #[pallet::weight(100_000)]
        pub fn convert_to_milestones(
            origin: OriginFor<T>,
        ) -> DispatchResultWithPostInfo {
			// Some method (that will eventually do the same thing as the brief evolver)
			// and allow for the submission of these milestones.
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
	}
	
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub enum VoteType {
		Approve,
		ChangesRequested,
		Cancel,
	}

	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub struct ProposedMilestoneWithInfo {
		percent: u8,
		ipfs_hash: [u8; 32],
	}

}
