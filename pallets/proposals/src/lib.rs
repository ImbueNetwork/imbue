#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use common_types::{CurrencyId, FundingType};
use frame_support::{
    pallet_prelude::*,
    storage::bounded_btree_map::BoundedBTreeMap,
    traits::{ConstU32, EnsureOrigin},
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::traits::AccountIdConversion;
use sp_std::{collections::btree_map::BTreeMap, convert::TryInto, prelude::*};

pub mod traits;
use traits::RefundHandler;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

pub mod migration;

pub mod impls;
pub use impls::*;

// The Constants associated with the bounded parameters
type MaxProjectKeysPerRound = ConstU32<1000>;
type MaxWhitelistPerProject = ConstU32<10000>;

pub type RoundKey = u32;
pub type ProjectKey = u32;
pub type MilestoneKey = u32;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

// These are the bounded types which are suitable for handling user input due to their restriction of vector length.
type BoundedProjectKeys = BoundedVec<ProjectKey, MaxProjectKeysPerRound>;
type BoundedMilestoneKeys<T> = BoundedVec<ProjectKey, <T as Config>::MaxMilestonesPerProject>;
pub type BoundedProposedMilestones<T> =
    BoundedVec<ProposedMilestone, <T as Config>::MaxMilestonesPerProject>;
pub type AgreementHash = H256;
type BoundedProjectKeysPerBlock<T> = BoundedVec<(ProjectKey, RoundType, MilestoneKey), <T as Config>::ExpiringProjectRoundsPerBlock>;
type ContributionsFor<T> = BTreeMap<AccountIdOf<T>, Contribution<BalanceOf<T>, BlockNumberFor<T>>>;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_identity::Config + pallet_timestamp::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type PalletId: Get<PalletId>;
        type AuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        type MultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        type WeightInfo: WeightInfo;
        type MaxWithdrawalExpiration: Get<Self::BlockNumber>;

        /// The amount of time given, up to point of decision, when a vote of no confidence is held.
        type NoConfidenceTimeLimit: Get<Self::BlockNumber>;

        /// The minimum percentage of votes, inclusive, that is required for a vote to pass.  
        // TODO: use percent
        type PercentRequiredForVoteToPass: Get<u8>;

        // TODO: use this.
        /// Maximum number of contributors per project.
        type MaximumContributorsPerProject: Get<u32>;

        /// Defines the length that a milestone can be voted on.
        type MilestoneVotingWindow: Get<Self::BlockNumber>;

        /// The type responisble for handling refunds.
        type RefundHandler: traits::RefundHandler<AccountIdOf<Self>, BalanceOf<Self>, CurrencyId>;

        type MaxMilestonesPerProject: Get<u32>;

        /// The storage deposit taken when a project is created and returned on deletion/completion.
        type ProjectStorageDeposit: Get<BalanceOf<Self>>;
        
        /// Imbue fee in percent 0-99
        //TODO: use percent.
        type ImbueFee: Get<u8>;

        /// The maximum projects to be dealt with per block. Must be small as is dealt with in the hooks.
        type ExpiringProjectRoundsPerBlock: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn projects)]
    pub type Projects<T: Config> = StorageMap<
        _,
        Identity,
        ProjectKey,
        Project<T::AccountId, BalanceOf<T>, T::BlockNumber>,
        OptionQuery,
    >;

    // TODO: MIGRATION NEEDE???? USER votes has been removed in favor of UserHasVoted.
    // This is because we need to clean up storage.
    // #[pallet::storage]
    // #[pallet::getter(fn user_votes)]
    // pub(super) type UserVotes<T: Config> = StorageMap<
    //     _,
    //     Identity,
    //     (T::AccountId, ProjectKey, MilestoneKey, RoundType),
    //     bool,
    //     ValueQuery,
    // >;

    // Bitmap of users that has voted, bounded by the number of contributors in a project.
    #[pallet::storage]
    pub(super) type UserHasVoted<T: Config> = StorageMap<_, Blake2_128, (ProjectKey, RoundType, MilestoneKey), BoundedBTreeMap<T::AccountId, bool, <T as Config>::MaximumContributorsPerProject>, ValueQuery>; 

    //TODO: Migration from storagemap to doublemap, also using blake 2 128 hasher
    #[pallet::storage]
    #[pallet::getter(fn milestone_votes)]  
    pub(super) type MilestoneVotes<T: Config> =
        StorageDoubleMap<_, Blake2_128, ProjectKey, Blake2_128, MilestoneKey, Vote<BalanceOf<T>>, OptionQuery>;

    /// This holds the votes when a no confidence round is raised.
    #[pallet::storage]
    #[pallet::getter(fn no_confidence_votes)]
    pub(super) type NoConfidenceVotes<T: Config> =
        StorageMap<_, Identity, ProjectKey, Vote<BalanceOf<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn project_count)]
    pub type ProjectCount<T> = StorageValue<_, ProjectKey, ValueQuery>;

    // TODO: STORAGE MIGRATION
    // everything has changed.
    /// Stores the ending block of the project key and round.
    #[pallet::storage]
    pub type Rounds<T> = StorageDoubleMap<_, Blake2_128, ProjectKey, Blake2_128, RoundType, BlockNumberFor<T>, OptionQuery>;

    /// Stores the project keys and round types ending on a given block
    #[pallet::storage]
    pub type RoundsExpiring<T> = StorageMap<_, Blake2_128, BlockNumberFor<T>, BoundedProjectKeysPerBlock<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn storage_version)]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Release, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// You have created a project.
        ProjectCreated(
            T::AccountId,
            H256,
            ProjectKey,
            BalanceOf<T>,
            common_types::CurrencyId,
            T::AccountId,
        ),
        /// A voting round has been created.
        VotingRoundCreated(ProjectKey),
        /// You have submitted a milestone.
        MilestoneSubmitted(T::AccountId, ProjectKey, MilestoneKey),
        /// A project has been cancelled.
        ProjectCancelled(ProjectKey),
        /// Successfully withdrawn funds from the project.
        ProjectFundsWithdrawn(T::AccountId, ProjectKey, BalanceOf<T>, CurrencyId),
        /// Vote submited successfully.
        VoteSubmitted(T::AccountId, ProjectKey, MilestoneKey, bool, T::BlockNumber),
        /// A milestone has been approved.
        MilestoneApproved(T::AccountId, ProjectKey, MilestoneKey, T::BlockNumber),
        /// You have created a vote of no confidence.
        NoConfidenceRoundCreated(ProjectKey),
        /// You have voted upon a round of no confidence.
        NoConfidenceRoundVotedUpon(ProjectKey),
        /// You have finalised a vote of no confidence.
        NoConfidenceRoundFinalised(ProjectKey),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Input parameter is invalid
        InvalidParam,
        /// There are no avaliable funds to withdraw.
        NoAvailableFundsToWithdraw,
        /// Project does not exist.
        ProjectDoesNotExist,
        /// Currently no active round to participate in.
        NoActiveRound,
        /// There was an overflow in pallet_proposals.
        Overflow,
        /// Only contributors can vote.
        OnlyContributorsCanVote,
        /// You do not have permission to do this.
        UserIsNotInitiator,
        /// The selected project does not exist in the round.
        ProjectNotInRound,
        /// The project has been cancelled.
        ProjectWithdrawn,
        /// Round has already started and cannot be modified.
        RoundStarted,
        /// Round has been cancelled.
        RoundCanceled,
        /// You have already voted on this round.
        VoteAlreadyExists,
        /// The voting threshhold has not been met.
        MilestoneVotingNotComplete,
        /// The given key must exist in storage.
        KeyNotFound,
        /// The voting threshold has not been met.
        VoteThresholdNotMet,
        /// The milestone does not exist.
        MilestoneDoesNotExist,
        /// You dont have enough IMBU for the project storage deposit.
        ImbueRequiredForStorageDep,
        /// Your account doenst have the privilage.
        InvalidAccount,
        /// The voting round has not started yet.
        VotingRoundNotStarted,
        /// you have already voted and cannot change your vote.
        VotesAreImmutable,
        /// The milestone has already been approved.
        MilestoneAlreadyApproved,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            // let mut weight = T::DbWeight::get().reads_writes(1, 1);
            // // Only supporting latest upgrade for now.
            // if StorageVersion::<T>::get() == Release::V2
            // {
            //     weight += migration::v3::migrate::<T>();
            //     StorageVersion::<T>::set(Release::V3);
            // }
            // weight
            Default::default()
        }

        // SAFETY: ExpiringProjectRoundsPerBlock has to be sane to prevent overweight blocks.
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 1);
            let key_type_vec = RoundsExpiring::<T>::take(n);

            key_type_vec.iter().for_each(|item| {
                let (project_key, round_type, milestone_key) = item;
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                
                // Remove the round prevents further voting.
                Rounds::<T>::remove(project_key, round_type);
                match round_type {
                    RoundType::VotingRound => {
                        // Since the threshold hasnt been met we must remove the vote to allow further
                        // milestone submission.
                        MilestoneVotes::<T>::remove(project_key, milestone_key);
                        UserHasVoted::<T>::remove((project_key, RoundType::VotingRound, milestone_key));
                    }
                    RoundType::VoteOfNoConfidence => {
                        // Here the vote can still be finalised, so do nothing for now
                        //TODO:
                    }
                }
            });

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Submit a milestones to be voted on.
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_milestone())]
        pub fn submit_milestone(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            milestone_key: MilestoneKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::new_milestone_submission(who, project_key, milestone_key)
        }

        /// The contributors call this to vote on a milestone submission.
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::vote_on_milestone())]
        pub fn vote_on_milestone(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            milestone_key: MilestoneKey,
            approve_milestone: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::new_milestone_vote(
                who,
                project_key,
                milestone_key,
                approve_milestone,
            )
        }

        /// Step 7 (INITATOR)
        /// Finalise the voting on a milestone.
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::finalise_milestone_voting())]
        pub fn finalise_milestone_voting(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            milestone_key: MilestoneKey,
        ) -> DispatchResultWithPostInfo {
            // Must be the initiator.
            let who = ensure_signed(origin)?;
            Self::do_finalise_milestone_voting(who, project_key, milestone_key)
        }

        /// Step 8 (INITATOR)
        /// Withdraw some avaliable funds from the project.
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::withdraw())]
        pub fn withdraw(
            origin: OriginFor<T>,
            project_key: ProjectKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::new_withdrawal(who, project_key)
        }

        /// In case of contributors losing confidence in the initiator a "Vote of no confidence" can be called.
        /// This will start a round which each contributor can vote on.
        /// The round will last as long as set in the Config.
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::raise_vote_of_no_confidence())]
        pub fn raise_vote_of_no_confidence(
            origin: OriginFor<T>,
            project_key: ProjectKey,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::raise_no_confidence_round(who, project_key)
        }

        /// Vote on an already existing "Vote of no condidence" round.
        /// is_yay is FOR the project's continuation.
        /// so is_yay = false == against the project from continuing perhaps should be flipped.
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::vote_on_no_confidence_round())]
        pub fn vote_on_no_confidence_round(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            is_yay: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::add_vote_no_confidence(who, project_key, is_yay)
        }

        /// Finalise a "vote of no condidence" round.
        /// Votes must pass a threshold as defined in the config trait for the vote to succeed.
        #[transactional]
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::finalise_no_confidence_round())]
        pub fn finalise_no_confidence_round(
            origin: OriginFor<T>,
            project_key: ProjectKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::call_finalise_no_confidence_vote(
                who,
                project_key,
                T::PercentRequiredForVoteToPass::get(),
            )
        }
    }
}


// TODO: MIGRATION NEEDED, removed field Contributionround
#[derive(Encode, Decode, PartialEq, Eq, Copy, Clone, Debug, TypeInfo)]
pub enum RoundType {
    VotingRound,
    VoteOfNoConfidence,
}

#[derive(Encode, Decode, TypeInfo, PartialEq)]
#[repr(u32)]
pub enum Release {
    V0,
    V1,
    V2,
    V3
}

impl Default for Release {
    fn default() -> Self {
        Self::V3
    }
}

/// The milestones provided by the user to define the milestones of a project.
/// TODO: add ipfs hash like in the grants pallet and
/// TODO: move these to a common repo (common_types will do)
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct ProposedMilestone {
    pub percentage_to_unlock: u32,
}

/// The contribution users made to a project project.
/// TODO: move these to a common repo (common_types will do)
/// TODO: add ipfs hash like in the grants pallet and
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct Milestone {
    pub project_key: ProjectKey,
    pub milestone_key: MilestoneKey,
    pub percentage_to_unlock: u32,
    pub is_approved: bool,
}

/// The vote struct is used to
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Vote<Balance> {
    yay: Balance,
    nay: Balance,
    is_approved: bool,
}

impl<Balance: From<u32>> Default for Vote<Balance> {
    fn default() -> Self {
        Self {
            yay: (0_u32).into(),
            nay: (0_u32).into(),
            is_approved: false,
        }
    }
}


// MIGRATION REQUIRED, REMOVED FIELDS: required_funds, approved_for_funding, funding_threshold_met
/// The struct that holds the descriptive properties of a project.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Project<AccountId, Balance, BlockNumber> {
    pub agreement_hash: H256,
    pub milestones: BTreeMap<MilestoneKey, Milestone>,
    pub contributions: BTreeMap<AccountId, Contribution<Balance, BlockNumber>>,
    pub currency_id: common_types::CurrencyId,
    pub withdrawn_funds: Balance,
    pub raised_funds: Balance,
    pub initiator: AccountId,
    pub created_on: BlockNumber,
    pub cancelled: bool,
    pub funding_type: FundingType,
}

/// The contribution users made to a proposal project.
/// TODO: Move to a common repo (common_types will do)
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct Contribution<Balance, BlockNumber> {
    /// Contribution value.
    pub value: Balance,
    /// Timestamp of the last contribution.
    pub timestamp: BlockNumber,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Whitelist<AccountId, Balance> {
    who: AccountId,
    max_cap: Balance,
}
