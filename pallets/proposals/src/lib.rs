#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use common_types::{CurrencyId, FundingType};
use frame_support::{
    dispatch::EncodeLike, pallet_prelude::*, storage::bounded_btree_map::BoundedBTreeMap,
    traits::EnsureOrigin, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
pub use pallet::*;
use pallet_deposits::traits::DepositHandler;
use scale_info::TypeInfo;
use sp_arithmetic::per_things::Percent;
use sp_core::H256;
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};
use sp_std::{collections::btree_map::*, convert::TryInto, prelude::*};

pub mod traits;
use traits::{IntoProposal, RefundHandler};

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod test_utils;

#[cfg(test)]
pub(crate) mod tests;

pub mod weights;
pub use weights::*;

pub mod migration;

pub mod impls;
pub use impls::*;
pub type ProjectKey = u32;
pub type MilestoneKey = u32;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
pub type StorageItemOf<T> =
    <<T as Config>::DepositHandler as DepositHandler<BalanceOf<T>, AccountIdOf<T>>>::StorageItem;
pub type DepositIdOf<T> =
    <<T as Config>::DepositHandler as DepositHandler<BalanceOf<T>, AccountIdOf<T>>>::DepositId;

// These are the bounded types which are suitable for handling user input due to their restriction of vector length.
type BoundedBTreeMilestones<T> =
    BoundedBTreeMap<MilestoneKey, Milestone, <T as Config>::MaxMilestonesPerProject>;
pub type BoundedProposedMilestones<T> =
    BoundedVec<ProposedMilestone, <T as Config>::MaxMilestonesPerProject>;
pub type AgreementHash = H256;
type BoundedProjectKeysPerBlock<T> =
    BoundedVec<(ProjectKey, RoundType, MilestoneKey), <T as Config>::ExpiringProjectRoundsPerBlock>;
type ContributionsFor<T> = BoundedBTreeMap<
    AccountIdOf<T>,
    Contribution<BalanceOf<T>, BlockNumberFor<T>>,
    <T as Config>::MaximumContributorsPerProject,
>;

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
        type WeightInfo: WeightInfoT;
        type MaxWithdrawalExpiration: Get<Self::BlockNumber>;
        /// The amount of time given, up to point of decision, when a vote of no confidence is held.
        type NoConfidenceTimeLimit: Get<Self::BlockNumber>;
        /// The minimum percentage of votes, inclusive, that is required for a vote to pass.  
        type PercentRequiredForVoteToPass: Get<Percent>;
        /// Maximum number of contributors per project.
        type MaximumContributorsPerProject: Get<u32>;
        /// Defines the length that a milestone can be voted on.
        type MilestoneVotingWindow: Get<Self::BlockNumber>;
        /// The type responisble for handling refunds.
        type RefundHandler: traits::RefundHandler<AccountIdOf<Self>, BalanceOf<Self>, CurrencyId>;
        type MaxMilestonesPerProject: Get<u32>;
        type MaxProjectsPerAccount: Get<u32>;
        /// Imbue fee in percent 0-99
        type ImbueFee: Get<Percent>;
        /// The maximum projects to be dealt with per block. Must be small as is dealt with in the hooks.
        type ExpiringProjectRoundsPerBlock: Get<u32>;
        /// The type responsible for storage deposits.
        type DepositHandler: DepositHandler<BalanceOf<Self>, AccountIdOf<Self>>;
        /// The type that will be used to calculate the deposit of a project.
        type ProjectStorageItem: Get<StorageItemOf<Self>>;
        /// The minimum percentage of votes, inclusive, that is required for a vote of no confidence to pass/finalize.
        type PercentRequiredForVoteNoConfidenceToPass: Get<Percent>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn projects)]
    pub type Projects<T: Config> = StorageMap<_, Identity, ProjectKey, Project<T>, OptionQuery>;

    // BTree of users that has voted, bounded by the number of contributors in a project.
    #[pallet::storage]
    pub(super) type UserHasVoted<T: Config> = StorageMap<
        _,
        Blake2_128,
        (ProjectKey, RoundType, MilestoneKey),
        BoundedBTreeMap<T::AccountId, bool, <T as Config>::MaximumContributorsPerProject>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn milestone_votes)]
    pub(super) type MilestoneVotes<T: Config> = StorageDoubleMap<
        _,
        Identity,
        ProjectKey,
        Identity,
        MilestoneKey,
        Vote<BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn completed_projects)]
    pub type CompletedProjects<T: Config> = StorageMap<
        _,
        Twox64Concat,
        AccountIdOf<T>,
        BoundedVec<ProjectKey, <T as Config>::MaxProjectsPerAccount>,
        ValueQuery,
    >;

    /// This holds the votes when a no confidence round is raised.
    #[pallet::storage]
    #[pallet::getter(fn no_confidence_votes)]
    pub(super) type NoConfidenceVotes<T: Config> =
        StorageMap<_, Identity, ProjectKey, Vote<BalanceOf<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn project_count)]
    pub type ProjectCount<T> = StorageValue<_, ProjectKey, ValueQuery>;

    /// Stores the ending block of the project key and round.
    #[pallet::storage]
    pub type Rounds<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProjectKey,
        Blake2_128Concat,
        RoundType,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    /// Stores the project keys and round types ending on a given block
    #[pallet::storage]
    pub type RoundsExpiring<T> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedProjectKeysPerBlock<T>,
        ValueQuery,
    >;

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
        NoConfidenceRoundCreated(T::AccountId, ProjectKey),
        /// You have voted upon a round of no confidence.
        NoConfidenceRoundVotedUpon(T::AccountId, ProjectKey),
        /// You have finalised a vote of no confidence.
        NoConfidenceRoundFinalised(T::AccountId, ProjectKey),
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
        /// There was an internal overflow prevented in pallet_proposals.
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
        /// Error with a mathematical operation
        MathError,
        /// There are too many contributions.
        TooManyContributions,
        /// There are too many milestones.
        TooManyMilestones,
        /// There are too many projects for a given account
        TooManyProjects,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// <HB SBP Review:
        ///
        /// I see this hook valid on testnet but if you will deploy this with weights v2 already, you can totally remove this.
        ///
        /// >
        fn on_runtime_upgrade() -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 1);
            // Only supporting latest upgrade for now.
            if StorageVersion::<T>::get() == Release::V2 {
                weight += migration::v3::migrate_all::<T>();
                StorageVersion::<T>::set(Release::V3);
            }
            weight
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
                    // Voting rounds automatically finalise if its reached its threshold.
                    // Therefore we can remove it on round end.
                    RoundType::VotingRound => {
                        weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));

                        MilestoneVotes::<T>::remove(project_key, milestone_key);
                        UserHasVoted::<T>::remove((
                            project_key,
                            RoundType::VotingRound,
                            milestone_key,
                        ));
                    }
                    // Votes of no confidence do not finaliese automatically
                    RoundType::VoteOfNoConfidence => {
                        // for now keep the round in tact and let them finalise.
                        // todo, this should be handled in pallet-dispute.
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
            Self::new_milestone_vote(who, project_key, milestone_key, approve_milestone)
        }

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

        /// pallet-disputes?
        /// Vote on an already existing "Vote of no condidence" round.
        /// is_yay is FOR the project's continuation.
        /// so is_yay == false == against the project from continuing.
        /// This autofinalises like in the milestone voting.
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
    }
    impl<T: crate::Config> IntoProposal<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>
        for crate::Pallet<T>
    where
        Project<T>: EncodeLike<Project<T>>,
    {
        /// The caller is used to take the storage deposit from.
        /// With briefs and grants the caller is the beneficiary, so the fee will come from them.
        fn convert_to_proposal(
            currency_id: CurrencyId,
            contributions: BTreeMap<AccountIdOf<T>, Contribution<BalanceOf<T>, BlockNumberFor<T>>>,
            brief_hash: H256,
            benificiary: AccountIdOf<T>,
            proposed_milestones: Vec<ProposedMilestone>,
            funding_type: FundingType,
        ) -> Result<(), DispatchError> {
            let project_key = crate::ProjectCount::<T>::get().saturating_add(1);

            // Take storage deposit only for a Project.
            let deposit_id = <T as Config>::DepositHandler::take_deposit(
                benificiary.clone(),
                <T as Config>::ProjectStorageItem::get(),
                CurrencyId::Native,
            )?;

            let sum_of_contributions = contributions
                .values()
                .fold(Default::default(), |acc: BalanceOf<T>, x| {
                    acc.saturating_add(x.value)
                });

            let project_account_id = crate::Pallet::<T>::project_account_id(project_key);

            match funding_type {
                FundingType::Proposal | FundingType::Brief => {
                    for (acc, cont) in contributions.iter() {
                        let project_account_id =
                            crate::Pallet::<T>::project_account_id(project_key);
                        <<T as Config>::MultiCurrency as MultiReservableCurrency<
                            AccountIdOf<T>,
                        >>::unreserve(currency_id, acc, cont.value);
                        <T as Config>::MultiCurrency::transfer(
                            currency_id,
                            acc,
                            &project_account_id,
                            cont.value,
                        )?;
                    }
                }
                FundingType::Grant(_) => {}
            }

            let mut milestone_key: u32 = 0;
            let mut milestones: BoundedBTreeMilestones<T> = BoundedBTreeMap::new();
            for milestone in proposed_milestones {
                let milestone = Milestone {
                    project_key,
                    milestone_key,
                    percentage_to_unlock: milestone.percentage_to_unlock,
                    is_approved: false,
                };
                milestones
                    .try_insert(milestone_key, milestone)
                    .map_err(|_| Error::<T>::TooManyMilestones)?;
                milestone_key = milestone_key.saturating_add(1);
            }

            let bounded_contributions: ContributionsFor<T> = contributions
                .try_into()
                .map_err(|_| Error::<T>::TooManyContributions)?;

            let project: Project<T> = Project {
                milestones,
                contributions: bounded_contributions,
                currency_id,
                withdrawn_funds: 0u32.into(),
                raised_funds: sum_of_contributions,
                initiator: benificiary.clone(),
                created_on: frame_system::Pallet::<T>::block_number(),
                cancelled: false,
                agreement_hash: brief_hash,
                funding_type,
                deposit_id,
            };

            Projects::<T>::insert(project_key, project);

            ProjectCount::<T>::mutate(|c| *c = c.saturating_add(1));

            Self::deposit_event(Event::ProjectCreated(
                benificiary,
                brief_hash,
                project_key,
                sum_of_contributions,
                currency_id,
                project_account_id,
            ));
            Ok(())
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Copy, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub enum RoundType {
    VotingRound,
    VoteOfNoConfidence,
}

#[derive(Encode, Decode, TypeInfo, PartialEq, MaxEncodedLen)]
#[repr(u32)]
pub enum Release {
    V0,
    V1,
    V2,
    V3,
    V4,
}

impl Default for Release {
    fn default() -> Self {
        Self::V3
    }
}

/// The milestones provided by the user to define the milestones of a project.
/// TODO: add ipfs hash like in the grants pallet and
/// TODO: move these to a common repo (common_types will do)
// MIGRATION! for briefs and grants
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct ProposedMilestone {
    pub percentage_to_unlock: Percent,
}

/// The contribution users made to a project project.
/// TODO: move these to a common repo (common_types will do)
/// TODO: add ipfs hash like in the grants pallet and
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct Milestone {
    pub project_key: ProjectKey,
    pub milestone_key: MilestoneKey,
    pub percentage_to_unlock: Percent,
    pub is_approved: bool,
}

/// The vote struct is used to
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct Vote<Balance> {
    yay: Balance,
    nay: Balance,
    is_approved: bool,
}

impl<Balance: From<u32>> Default for Vote<Balance> {
    fn default() -> Self {
        Self {
            yay: Balance::from(Zero::zero()),
            nay: Balance::from(Zero::zero()),
            is_approved: false,
        }
    }
}

/// The struct which contain milestones that can be submitted.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Project<T: Config> {
    pub agreement_hash: H256,
    pub milestones: BoundedBTreeMilestones<T>,
    pub contributions: ContributionsFor<T>,
    pub currency_id: common_types::CurrencyId,
    pub withdrawn_funds: BalanceOf<T>,
    pub raised_funds: BalanceOf<T>,
    pub initiator: AccountIdOf<T>,
    pub created_on: BlockNumberFor<T>,
    pub cancelled: bool,
    pub funding_type: FundingType,
    pub deposit_id: DepositIdOf<T>,
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

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct Whitelist<AccountId, Balance> {
    who: AccountId,
    max_cap: Balance,
}

pub trait WeightInfoT {
    fn submit_milestone() -> Weight;
    fn vote_on_milestone() -> Weight;
    fn withdraw() -> Weight;
    fn raise_vote_of_no_confidence() -> Weight;
    fn vote_on_no_confidence_round() -> Weight;
}
