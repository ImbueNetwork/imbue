#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use common_types::CurrencyId;
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
type MaxMilestoneKeys = ConstU32<100>;
type MaxProposedMilestones = ConstU32<100>;
type MaxWhitelistPerProject = ConstU32<10000>;

pub type RoundKey = u32;
pub type ProjectKey = u32;
pub type MilestoneKey = u32;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
type RoundOf<T> = Round<<T as frame_system::Config>::BlockNumber>;
pub type TimestampOf<T> = <T as pallet_timestamp::Config>::Moment;
pub type ProjectAccountId<T> = <T as frame_system::Config>::AccountId;
pub type Refunds<T> = Vec<(
    AccountIdOf<T>,
    ProjectAccountId<T>,
    BalanceOf<T>,
    CurrencyId,
)>;
// These are the bounded types which are suitable for handling user input due to their restriction of vector length.
type BoundedWhitelistSpots<T> =
    BoundedBTreeMap<AccountIdOf<T>, BalanceOf<T>, MaxWhitelistPerProject>;
type BoundedProjectKeys = BoundedVec<ProjectKey, MaxProjectKeysPerRound>;
type BoundedMilestoneKeys = BoundedVec<ProjectKey, MaxMilestoneKeys>;
pub type BoundedProposedMilestones = BoundedVec<ProposedMilestone, MaxProposedMilestones>;
pub type AgreementHash = H256;

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

        type MaxProjectsPerRound: Get<u32>;

        type MaxWithdrawalExpiration: Get<Self::BlockNumber>;

        type WeightInfo: WeightInfo;

        /// The amount of time given, up to point of decision, when a vote of no confidence is held.
        type NoConfidenceTimeLimit: Get<Self::BlockNumber>;

        /// The minimum percentage of votes, inclusive, that is required for a vote to pass.  
        type PercentRequiredForVoteToPass: Get<u8>;

        /// Maximum number of contributors per project.
        type MaximumContributorsPerProject: Get<u32>;

        /// Defines the amount of refunds that occur in the on initialise method.
        /// Does not include the remaining refunds that may occur in the on_idle hook.
        type RefundsPerBlock: Get<u8>;
    }

    #[pallet::type_value]
    pub fn InitialMilestoneVotingWindow() -> u32 {
        100800u32
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
        Project<T::AccountId, BalanceOf<T>, T::BlockNumber, TimestampOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn whitelist_spots)]
    pub type WhitelistSpots<T: Config> =
        StorageMap<_, Identity, ProjectKey, BTreeMap<T::AccountId, BalanceOf<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn user_votes)]
    pub(super) type UserVotes<T: Config> = StorageMap<
        _,
        Identity,
        (T::AccountId, ProjectKey, MilestoneKey, RoundKey),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn milestone_votes)]
    pub(super) type MilestoneVotes<T: Config> =
        StorageMap<_, Identity, (ProjectKey, MilestoneKey), Vote<BalanceOf<T>>, OptionQuery>;

    /// This holds the votes when a no confidence round is raised.
    #[pallet::storage]
    #[pallet::getter(fn no_confidence_votes)]
    pub(super) type NoConfidenceVotes<T: Config> =
        StorageMap<_, Identity, ProjectKey, Vote<BalanceOf<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn project_count)]
    pub type ProjectCount<T> = StorageValue<_, ProjectKey, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn rounds)]
    pub type Rounds<T> = StorageMap<_, Identity, RoundKey, Option<RoundOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn round_count)]
    pub type RoundCount<T> = StorageValue<_, RoundKey, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn max_project_count_per_round)]
    pub type MaxProjectCountPerRound<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn milestone_voting_window)]
    pub type MilestoneVotingWindow<T> =
        StorageValue<_, u32, ValueQuery, InitialMilestoneVotingWindow>;

    #[pallet::storage]
    #[pallet::getter(fn withdrawal_expiration)]
    pub type WithdrawalExpiration<T> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_identity_required)]
    pub type IsIdentityRequired<T> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn storage_version)]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Release, ValueQuery>;

    /// The refund queue is used to store the list of accounts that have been involved in a do_refund call.
    /// The queue will be sent to the hooks which will inturn actually carry out the
    #[pallet::storage]
    #[pallet::getter(fn refund_queue)]
    pub type RefundQueue<T> = StorageValue<
        _,
        Vec<(
            AccountIdOf<T>,
            ProjectAccountId<T>,
            BalanceOf<T>,
            CurrencyId,
        )>,
        ValueQuery,
    >;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
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
        ),
        // Project has been updated
        ProjectUpdated(T::AccountId, ProjectKey, BalanceOf<T>),
        /// A funding round has been created.
        FundingRoundCreated(RoundKey, Vec<ProjectKey>),
        /// A voting round has been created.
        VotingRoundCreated(RoundKey, Vec<ProjectKey>),
        /// You have submitted a milestone.
        MilestoneSubmitted(T::AccountId, ProjectKey, MilestoneKey),
        /// Contribution has succeded.
        ContributeSucceeded(
            T::AccountId,
            ProjectKey,
            BalanceOf<T>,
            common_types::CurrencyId,
            T::BlockNumber,
        ),
        /// A project has been cancelled.
        ProjectCancelled(RoundKey, ProjectKey),
        /// Successfully withdrawn funds from the project.
        ProjectFundsWithdrawn(T::AccountId, ProjectKey, BalanceOf<T>, CurrencyId),
        /// A project has been approved.
        ProjectApproved(RoundKey, ProjectKey),
        /// A round has been cancelled.
        RoundCancelled(RoundKey),
        /// Vote submited successfully.
        VoteComplete(T::AccountId, ProjectKey, MilestoneKey, bool, T::BlockNumber),
        /// A milestone has been approved.
        MilestoneApproved(T::AccountId, ProjectKey, MilestoneKey, T::BlockNumber),
        /// A white list has been added.
        WhitelistAdded(ProjectKey, T::BlockNumber),
        /// A white list has been removed.
        WhitelistRemoved(ProjectKey, T::BlockNumber),
        /// A project has been added to refund queue.
        ProjectFundsAddedToRefundQueue(ProjectKey, BalanceOf<T>),
        /// You have created a vote of no confidence.
        NoConfidenceRoundCreated(RoundKey, ProjectKey),
        /// You have voted upon a round of no confidence.
        NoConfidenceRoundVotedUpon(RoundKey, ProjectKey),
        /// You have finalised a vote of no confidence.
        NoConfidenceRoundFinalised(RoundKey, ProjectKey),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Contribution has exceeded the maximum capacity of the project.
        ContributionMustBeLowerThanMaxCap,
        /// This block number must be later than the current.
        EndBlockNumberInvalid,
        /// The starting block number must be before the ending block number.
        EndTooEarly,
        /// Required identity not found.
        IdentityNeeded,
        /// Input parameter is invalid
        InvalidParam,
        /// There are no avaliable funds to withdraw.
        NoAvailableFundsToWithdraw,
        /// Your account does not have the correct authority.
        InvalidAccount,
        /// Project does not exist.
        ProjectDoesNotExist,
        /// Project name is a mandatory field.
        ProjectNameIsMandatory,
        /// Project logo is a mandatory field.
        LogoIsMandatory,
        /// Project description is a mandatory field.
        ProjectDescriptionIsMandatory,
        /// Website url is a mandatory field.
        WebsiteURLIsMandatory,
        /// Milestones totals do not add up to 100%.
        MilestonesTotalPercentageMustEqual100,
        MilestoneDoesNotExist,
        /// Currently no active round to participate in.
        NoActiveRound,
        // TODO: NOT IN USE
        NoActiveProject,
        /// There was an overflow.
        Overflow,
        /// A project must be approved before the submission of milestones.
        OnlyApprovedProjectsCanSubmitMilestones,
        /// Only contributors can vote.
        OnlyContributorsCanVote,
        /// You do not have permission to do this.
        UserIsNotInitiator,
        /// You do not have permission to do this.
        OnlyInitiatorOrAdminCanApproveMilestone,
        /// You do not have permission to do this.
        OnlyWhitelistedAccountsCanContribute,
        // TODO: not in use
        ProjectAmountExceed,
        /// The selected project does not exist in the round.
        ProjectNotInRound,
        /// The project has been cancelled.
        ProjectWithdrawn,
        // TODO: not in use.
        ProjectApproved,
        /// Parameter limit exceeded.
        ParamLimitExceed,
        /// Round has already started and cannot be modified.
        RoundStarted,
        /// Round stll in progress.
        RoundNotEnded,
        /// Round has been cancelled.
        RoundCanceled,
        /// The start block number is invalid.
        StartBlockNumberInvalid,
        /// You have already voted on this round.
        VoteAlreadyExists,
        /// The voting threshhold has not been met.
        MilestoneVotingNotComplete,
        // TODO: not in use
        WithdrawalExpirationExceed,
        /// The given key must exist in storage.
        KeyNotFound,
        /// The input vector must exceed length zero.
        LengthMustExceedZero,
        /// The voting threshold has not been met.
        VoteThresholdNotMet,
        /// The project must be approved.
        ProjectApprovalRequired,
        /// The round type specified is invalid.
        InvalidRoundType,
        /// The project already be approved, cannot be updated.
        ProjectAlreadyApproved,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 1);
            if StorageVersion::<T>::get() == Release::V0
                || StorageVersion::<T>::get() == Release::V1
            {
                weight += migration::v2::migrate::<T>();
                StorageVersion::<T>::set(Release::V2);
            }
            weight
        }

        fn on_initialize(_b: T::BlockNumber) -> Weight {
            let mut weight = Weight::default();

            let mut refunds = RefundQueue::<T>::get();
            // Overestimate.
            weight += T::DbWeight::get().reads(2);

            // A counter is used to know how many elements to split off.
            let mut c = 0u32;
            for i in 0..T::RefundsPerBlock::get() {
                if let Some(refund) = refunds.get(i as usize) {
                    Self::refund_item_in_queue(&refund.1, &refund.0, refund.2, refund.3);
                    weight
                        .saturating_add(<T as pallet::Config>::WeightInfo::refund_item_in_queue());
                    c += 1;
                } else {
                    break;
                }
            }

            Self::split_off_refunds(&mut refunds, c);
            weight.saturating_add(<T as pallet::Config>::WeightInfo::split_off_refunds());

            weight
        }

        fn on_idle(_b: T::BlockNumber, remaining_weight: Weight) -> Weight {
            let mut refunds = RefundQueue::<T>::get();
            // Overestimate.
            remaining_weight.saturating_sub(T::DbWeight::get().reads(2));

            // A little extra than required for safety.
            let weight_required_to_finish_hook =
                <T as pallet::Config>::WeightInfo::refund_item_in_queue()
                    + <T as pallet::Config>::WeightInfo::split_off_refunds();

            // A counter is used to know how many elements to split off.
            let mut c = 0u32;

            // While the weight has enough weight to finish off the
            while remaining_weight.ref_time() > weight_required_to_finish_hook.ref_time() {
                if let Some(refund) = refunds.get(c as usize) {
                    let _ = Self::refund_item_in_queue(&refund.1, &refund.0, refund.2, refund.3);
                    remaining_weight
                        .saturating_sub(<T as pallet::Config>::WeightInfo::refund_item_in_queue());
                    c += 1;
                } else {
                    break;
                }
            }

            let _ = Self::split_off_refunds(&mut refunds, c);
            remaining_weight.saturating_sub(<T as pallet::Config>::WeightInfo::split_off_refunds());

            remaining_weight
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Step 1 (INITIATOR)
        /// Create project.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_project())]
        pub fn create_project(
            origin: OriginFor<T>,
            agreement_hash: H256,
            proposed_milestones: BoundedProposedMilestones,
            required_funds: BalanceOf<T>,
            currency_id: common_types::CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Validation
            let mut total_percentage = 0;
            for milestone in proposed_milestones.iter() {
                total_percentage += milestone.percentage_to_unlock;
            }
            ensure!(
                total_percentage == 100,
                Error::<T>::MilestonesTotalPercentageMustEqual100
            );

            Self::new_project(
                who,
                agreement_hash,
                proposed_milestones,
                required_funds,
                currency_id,
            )
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_project())]
        pub fn update_project(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            proposed_milestones: BoundedProposedMilestones,
            required_funds: BalanceOf<T>,
            currency_id: CurrencyId,
            agreement_hash: H256,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let mut total_percentage = 0;
            for milestone in proposed_milestones.iter() {
                total_percentage += milestone.percentage_to_unlock;
            }
            ensure!(
                total_percentage == 100,
                Error::<T>::MilestonesTotalPercentageMustEqual100
            );

            Self::update_existing_project(
                who,
                project_key,
                proposed_milestones,
                required_funds,
                currency_id,
                agreement_hash,
            )
            Self::deposit_event(Event::ProjectUpdated(who, project_key, required_funds));
        }

        /// Step 1.5 (INITIATOR)
        /// Add whitelist to a project
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::add_project_whitelist())]
        pub fn add_project_whitelist(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            new_whitelist_spots: BoundedWhitelistSpots<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_initiator(who, project_key)?;
            let mut project_whitelist_spots =
                WhitelistSpots::<T>::get(project_key).unwrap_or(BTreeMap::new());
            project_whitelist_spots.extend(new_whitelist_spots);
            <WhitelistSpots<T>>::insert(project_key, project_whitelist_spots);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::WhitelistAdded(project_key, now));
            Ok(().into())
        }

        /// Step 1.5 (INITIATOR)
        /// Remove a whitelist
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_project_whitelist())]
        pub fn remove_project_whitelist(
            origin: OriginFor<T>,
            project_key: ProjectKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_initiator(who, project_key)?;
            <WhitelistSpots<T>>::remove(project_key);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::WhitelistRemoved(project_key, now));
            Ok(().into())
        }

        /// Step 2 (ADMIN)
        /// Schedule a round
        /// project_keys: the projects were selected for this round
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::schedule_round())]
        pub fn schedule_round(
            origin: OriginFor<T>,
            start: T::BlockNumber,
            end: T::BlockNumber,
            project_keys: BoundedProjectKeys,
            round_type: RoundType,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            let now = <frame_system::Pallet<T>>::block_number();
            // The end block must be greater than the start block
            ensure!(end > start, Error::<T>::EndTooEarly);
            // Both the starting block number and the ending block number must be greater than the current number of blocks
            ensure!(start >= now, Error::<T>::StartBlockNumberInvalid);
            ensure!(end > now, Error::<T>::EndBlockNumberInvalid);
            ensure!(!project_keys.is_empty(), Error::<T>::LengthMustExceedZero);

            // Project keys is bounded to 5 projects maximum.
            let max_project_key = project_keys
                .iter()
                .max()
                .ok_or(Error::<T>::LengthMustExceedZero)?;
            Projects::<T>::get(max_project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
            Self::new_round(start, end, project_keys, round_type)
        }

        /// Step 2.5 (ADMIN)
        /// Cancel a round
        /// This round must have not started yet
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_round())]
        pub fn cancel_round(
            origin: OriginFor<T>,
            round_key: RoundKey,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            let now = <frame_system::Pallet<T>>::block_number();
            let mut round = <Rounds<T>>::get(round_key).ok_or(Error::<T>::NoActiveRound)?;

            // Ensure current round is not started
            ensure!(round.start > now, Error::<T>::RoundStarted);
            // This round cannot be cancelled
            ensure!(!round.is_canceled, Error::<T>::RoundCanceled);

            round.is_canceled = true;
            <Rounds<T>>::insert(round_key, Some(round));

            Self::deposit_event(Event::RoundCancelled(round_key));

            Ok(().into())
        }

        /// Step 3 (CONTRIBUTOR/FUNDER)
        /// Contribute to a project
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::contribute())]
        #[transactional]
        pub fn contribute(
            origin: OriginFor<T>,
            round_key: Option<RoundKey>,
            project_key: ProjectKey,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let contribution_round_key = round_key.unwrap_or(RoundCount::<T>::get());
            Self::new_contribution(who, contribution_round_key, project_key, value)
        }

        /// Step 4 (ADMIN)
        /// Approve project
        /// If the project is approved, the project initiator can withdraw funds for approved milestones
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::approve())]
        pub fn approve(
            origin: OriginFor<T>,
            round_key: Option<RoundKey>,
            project_key: ProjectKey,
            milestone_keys: Option<BoundedMilestoneKeys>,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            let approval_round_key = round_key.unwrap_or(RoundCount::<T>::get());
            Self::do_approve(project_key, approval_round_key, milestone_keys)
        }

        /// Step 5 (INITIATOR)
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

        /// Step 6 (CONTRIBUTOR/FUNDER)
        /// Vote on a milestone
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::vote_on_milestone())]
        pub fn vote_on_milestone(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            milestone_key: MilestoneKey,
            round_key: Option<RoundKey>,
            approve_milestone: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let voting_round_key = round_key.unwrap_or(RoundCount::<T>::get());
            Self::new_milestone_vote(
                who,
                project_key,
                milestone_key,
                voting_round_key,
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
            round_key: Option<RoundKey>,
            project_key: ProjectKey,
            is_yay: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let voting_round_key = round_key.unwrap_or(RoundCount::<T>::get());
            Self::add_vote_no_confidence(who, voting_round_key, project_key, is_yay)
        }

        /// Finalise a "vote of no condidence" round.
        /// Votes must pass a threshold as defined in the config trait for the vote to succeed.
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::finalise_no_confidence_round())]
        pub fn finalise_no_confidence_round(
            origin: OriginFor<T>,
            round_key: Option<RoundKey>,
            project_key: ProjectKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let voting_round_key = round_key.unwrap_or(RoundCount::<T>::get());
            Self::call_finalise_no_confidence_vote(
                who,
                voting_round_key,
                project_key,
                T::PercentRequiredForVoteToPass::get(),
            )
        }

        // Root Extrinsics:

        /// Set max project count per round
        #[pallet::call_index(15)]
        #[pallet::weight(<T as Config>::WeightInfo::set_storage_variable())]
        pub fn set_max_project_count_per_round(
            origin: OriginFor<T>,
            max_project_count_per_round: u32,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            ensure!(
                max_project_count_per_round > 0,
                Error::<T>::ParamLimitExceed
            );
            MaxProjectCountPerRound::<T>::put(max_project_count_per_round);

            Ok(().into())
        }

        /// Set milestone voting window
        #[pallet::call_index(16)]
        #[pallet::weight(<T as Config>::WeightInfo::set_storage_variable())]
        pub fn set_milestone_voting_window(
            origin: OriginFor<T>,
            new_milestone_voting_window: u32,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            ensure!(
                new_milestone_voting_window > 0,
                Error::<T>::ParamLimitExceed
            );
            MilestoneVotingWindow::<T>::put(new_milestone_voting_window);

            Ok(().into())
        }

        /// Set withdrawal expiration
        #[pallet::call_index(17)]
        #[pallet::weight(<T as Config>::WeightInfo::set_storage_variable())]
        pub fn set_withdrawal_expiration(
            origin: OriginFor<T>,
            withdrawal_expiration: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            ensure!(
                withdrawal_expiration > (0_u32).into(),
                Error::<T>::InvalidParam
            );
            <WithdrawalExpiration<T>>::put(withdrawal_expiration);

            Ok(().into())
        }

        /// set is_identity_required
        #[pallet::call_index(18)]
        #[pallet::weight(<T as Config>::WeightInfo::set_storage_variable())]
        pub fn set_is_identity_required(
            origin: OriginFor<T>,
            is_identity_required: bool,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            IsIdentityRequired::<T>::put(is_identity_required);

            Ok(().into())
        }

        /// Ad Hoc Step (ADMIN)
        /// This will add the refunds to a queue to eventually be processed, Hooks will show refunds themselves.
        #[pallet::call_index(19)]
        #[pallet::weight(<T as Config>::WeightInfo::refund())]
        pub fn refund(origin: OriginFor<T>, project_key: ProjectKey) -> DispatchResultWithPostInfo {
            //ensure only admin can perform refund
            T::AuthorityOrigin::ensure_origin(origin)?;
            Self::add_refunds_to_queue(project_key)
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub enum RoundType {
    ContributionRound,
    VotingRound,
    VoteOfNoConfidence,
}

#[derive(Encode, Decode, TypeInfo, PartialEq)]
#[repr(u32)]
pub enum Release {
    V0,
    V1,
    V2,
}

impl Default for Release {
    fn default() -> Self {
        Self::V0
    }
}

/// The round struct contains all the data associated with a given round.
/// A round may include multiple projects.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Round<BlockNumber> {
    start: BlockNumber,
    end: BlockNumber,
    project_keys: Vec<ProjectKey>,
    round_type: RoundType,
    is_canceled: bool,
}

impl<BlockNumber: From<u32>> Round<BlockNumber> {
    fn new(
        start: BlockNumber,
        end: BlockNumber,
        project_keys: Vec<ProjectKey>,
        round_type: RoundType,
    ) -> Round<BlockNumber> {
        Round {
            start,
            end,
            project_keys,
            is_canceled: false,
            round_type,
        }
    }
}

/// The contribution users made to a project project.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct ProposedMilestone {
    pub percentage_to_unlock: u32,
}

/// The contribution users made to a project project.
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

/// The struct that holds the descriptive properties of a project.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Project<AccountId, Balance, BlockNumber, Timestamp> {
    pub agreement_hash: H256,
    pub milestones: BTreeMap<MilestoneKey, Milestone>,
    pub contributions: BTreeMap<AccountId, Contribution<Balance, Timestamp>>,
    pub currency_id: common_types::CurrencyId,
    pub required_funds: Balance,
    pub withdrawn_funds: Balance,
    pub raised_funds: Balance,
    pub initiator: AccountId,
    pub created_on: BlockNumber,
    pub approved_for_funding: bool,
    pub funding_threshold_met: bool,
    pub cancelled: bool,
}

/// The contribution users made to a proposal project.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct Contribution<Balance, Timestamp> {
    /// Contribution value.
    pub value: Balance,
    /// Timestamp of the last contribution.
    pub timestamp: Timestamp,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Whitelist<AccountId, Balance> {
    who: AccountId,
    max_cap: Balance,
}
