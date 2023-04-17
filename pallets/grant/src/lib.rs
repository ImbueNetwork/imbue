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
		type MaxApprovers: Get<u32>;
	
		// Used to remove ignored grants and keep a clean system.
		type GrantVotingPeriod: Get<<Self as frame_system::Config>::BlockNumber>;
	}

	#[pallet::storage]
	pub type PendingGrants<T: Config> = StorageMap<_, Blake2_128, T::GrantId, Grant<T>, OptionQuery>;

	//#[pallet::storage]
	//#[pallet::getter(fn something)]
	//pub type GrantVotes<T: Config> = StorageMap<_, Blake2_128, (T::GrantId), BTreeMap>;

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
				approvers: assigned_approvers
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

		/// For the people approving, they must submit the vote to accept or decline the grant proposal. 
		#[pallet::call_index(1)]
        #[pallet::weight(100_000)]
        pub fn vote_on_grant(
            origin: OriginFor<T>,
			vote: VoteType,
			grant_id: T::GrantId,
        ) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
			let grant: Grant<T> = PendingGrants::<T>::get(grant_id).ok_or(Error::<T>::GrantNotFound)?;

			ensure!(grant.approvers.iter().any(|approver|approver == &voter), Error::<T>::OnlyApproversCanVote);
			// generate a vote key
			// insert into vote storage
			// add hook to see if at the start of the block there is a super majority and close.

            Self::deposit_event(Event::<T>::GrantVotedUpon{voter, grant_id, way: vote});
            Ok(().into())
        }

		#[pallet::call_index(2)]
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
