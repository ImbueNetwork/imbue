#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use common_types::CurrencyId;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

use frame_support::{
    pallet_prelude::*,
    transactional,
    PalletId, 
    traits::ConstU32
    };
use orml_traits::MultiCurrency;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::traits::AccountIdConversion;
use sp_std::{
    convert::TryInto,
    prelude::*,
    vec
};
use frame_system::pallet_prelude::*;
#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_identity::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type PalletId: Get<PalletId>;

        type MultiCurrency: MultiCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;

        type MaxProposalsPerRound: Get<u32>;

        type MaxWithdrawalExpiration: Get<Self::BlockNumber>;

        type WeightInfo: WeightInfo;

        /// The amount of time given ,up to point of decision, when a vote of no confidence is held.
        type NoConfidenceTimeLimit: Get<Self::BlockNumber>;
    }


    #[pallet::type_value]
    pub fn InitialMilestoneVotingWindow() -> u32
    {
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
        Project<T::AccountId, BalanceOf<T>, T::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn whitelist_spots)]
    pub type WhitelistSpots<T: Config> = StorageMap<
        _,
        Identity,
        ProjectKey,
        Vec<Whitelist<T::AccountId, BalanceOf<T>>>,
        OptionQuery,
    >;

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
    #[pallet::getter(fn max_proposal_count_per_round)]
    pub type MaxProposalCountPerRound<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn milestone_voting_window)]
    pub type MilestoneVotingWindow<T> = StorageValue<_, u32, ValueQuery, InitialMilestoneVotingWindow>;

    #[pallet::storage]
    #[pallet::getter(fn withdrawal_expiration)]
    pub type WithdrawalExpiration<T> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_identity_required)]
    pub type IsIdentityRequired<T> = StorageValue<_, bool, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProjectCreated(
            T::AccountId,
            Vec<u8>,
            ProjectKey,
            BalanceOf<T>,
            common_types::CurrencyId,
        ),
        FundingRoundCreated(RoundKey, Vec<ProjectKey>),
        VotingRoundCreated(RoundKey, Vec<ProjectKey>),
        MilestoneSubmitted(T::AccountId, ProjectKey, MilestoneKey),
        ContributeSucceeded(
            T::AccountId,
            ProjectKey,
            BalanceOf<T>,
            common_types::CurrencyId,
            T::BlockNumber,
        ),
        ProjectCancelled(RoundKey, ProjectKey),
        ProjectFundsWithdrawn(T::AccountId, ProjectKey, BalanceOf<T>, CurrencyId),
        ProjectApproved(RoundKey, ProjectKey),
        RoundCancelled(RoundKey),
        VoteComplete(T::AccountId, ProjectKey, MilestoneKey, bool, T::BlockNumber),
        MilestoneApproved(T::AccountId, ProjectKey, MilestoneKey, T::BlockNumber),
        WhitelistAdded(ProjectKey, T::BlockNumber),
        WhitelistRemoved(ProjectKey, T::BlockNumber),
        ProjectLockedFundsRefunded(ProjectKey, BalanceOf<T>),
        NoConfidenceRoundCreated(RoundKey, Vec<ProjectKey>),
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
        /// Project name is a mandatory field. 
        LogoIsMandatory,
        /// Project name is a mandatory field. 
        ProjectDescriptionIsMandatory,
        /// Project name is a mandatory field. 
        WebsiteURLIsMandatory,
        /// Milestones do not add up to 100%.
        MilestonesTotalPercentageMustEqual100,
        /// Currently no active round to participate in.
        NoActiveRound,
        /// TODO: NOT IN USE
        NoActiveProposal,
        /// There was an overflow.
        Overflow,
        OnlyApprovedProjectsCanSubmitMilestones,
        OnlyContributorsCanVote,
        UserIsNotInitator,
        OnlyInitiatorOrAdminCanApproveMilestone,
        OnlyWhitelistedAccountsCanContribute,
        ProposalAmountExceed,
        ProjectNotInRound,
        ProposalWithdrawn,
        ProposalApproved,
        ParamLimitExceed,
        RoundStarted,
        RoundNotEnded,
        RoundNotProcessing,
        RoundCanceled,
        /// Errors should have helpful documentation associated with them.
        StartBlockNumberTooSmall,
        VoteAlreadyExists,
        MilestoneVotingNotComplete,
        WithdrawalExpirationExceed,
        /// The given key must exist in storage.
        KeyNotFound,
        /// The input vector must exceed length zero.
        LengthMustExceedZero,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Step 1 (INITATOR)
        /// Create project.
        #[pallet::weight(<T as Config>::WeightInfo::create_project())]
        pub fn create_project(
            origin: OriginFor<T>,
            name: BoundedStringField,
            logo: BoundedStringField,
            description: BoundedDescriptionField,
            website: BoundedDescriptionField,
            proposed_milestones: BoundedProposedMilestones,
            required_funds: BalanceOf<T>,
            currency_id: common_types::CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::new_project(
                who,
                name,
                logo,
                description,
                website,
                proposed_milestones,
                required_funds,
                currency_id,
            )
        }


        /// Step 1.5 (INITATOR)
        /// Add whitelist to a project
        #[pallet::weight(<T as Config>::WeightInfo::create_project())]
        pub fn add_project_whitelist(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            whitelist_spots: BoundedWhitelistSpots<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_initator(who, project_key)?;
            let mut project_whitelist_spots: Vec<Whitelist<AccountIdOf<T>, BalanceOf<T>>> =
                Vec::new();

            let whitelist_exists = WhitelistSpots::<T>::contains_key(project_key);
            if whitelist_exists {
                let existing_spots = Self::whitelist_spots(project_key).ok_or(Error::<T>::KeyNotFound)?;
                project_whitelist_spots.extend(existing_spots);
            }

            project_whitelist_spots.extend(whitelist_spots);
            <WhitelistSpots<T>>::insert(project_key, project_whitelist_spots);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::WhitelistAdded(project_key, now));
            Ok(().into())
        }

        /// Step 1.5 (INITATOR)
        /// Remove a whitelist
        #[pallet::weight(<T as Config>::WeightInfo::create_project())]
        pub fn remove_project_whitelist(
            origin: OriginFor<T>,
            project_key: ProjectKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_initator(who, project_key)?;
            <WhitelistSpots<T>>::remove(project_key);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::WhitelistRemoved(project_key, now));
            Ok(().into())
        }

        /// Step 2 (ADMIN)
        /// Schedule a round
        /// proposal_keys: the proposals were selected for this round
        #[pallet::weight(<T as Config>::WeightInfo::schedule_round(MaxProposalCountPerRound::<T>::get()))]
        pub fn schedule_round(
            origin: OriginFor<T>,
            start: T::BlockNumber,
            end: T::BlockNumber,
            project_keys: BoundedProjectKeys,
            round_type: RoundType
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::new_round(start, end, project_keys, round_type)
        }

        /// Step 2.5 (ADMIN)
        /// Cancel a round
        /// This round must have not started yet
        /// TODO: BUG currently since we can have multpile projects in a round if root deletes a key with someone elses project in then both are deleted.
        // WRITE TEST
        #[pallet::weight(<T as Config>::WeightInfo::cancel_round())]
        pub fn cancel_round(
            origin: OriginFor<T>,
            round_key: RoundKey,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let now = <frame_system::Pallet<T>>::block_number();
            let count = RoundCount::<T>::get();
            let mut round = <Rounds<T>>::get(round_key).ok_or(Error::<T>::NoActiveRound)?;

            // Ensure current round is not started
            ensure!(round.start > now, Error::<T>::RoundStarted);
            // This round cannot be cancelled
            ensure!(!round.is_canceled, Error::<T>::RoundCanceled);

            round.is_canceled = true;
            <Rounds<T>>::insert(round_key, Some(round));

            // TODO loop through projects and refund contributers

            Self::deposit_event(Event::RoundCancelled(count - 1));

            Ok(().into())
        }

        /// Step 3 (CONTRIBUTOR/FUNDER)
        /// Contribute to a proposal
        #[pallet::weight(<T as Config>::WeightInfo::contribute())]
        #[transactional]
        pub fn contribute(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::new_contribution(who, project_key, value)
        }

        /// Step 4 (ADMIN)
        /// Approve project
        /// If the project is approved, the project initator can withdraw funds for approved milestones
        #[pallet::weight(<T as Config>::WeightInfo::approve())]
        pub fn approve(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            milestone_keys: Option<BoundedMilestoneKeys>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::do_approve(project_key, milestone_keys)
        }

        /// Step 5 (INITATOR)
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
        #[pallet::weight(<T as Config>::WeightInfo::contribute())]
        pub fn vote_on_milestone(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            milestone_key: MilestoneKey,
            approve_milestone: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::new_milestone_vote(who, project_key, milestone_key, approve_milestone)
        }

        /// Step 7 (INITATOR)
        #[pallet::weight(<T as Config>::WeightInfo::submit_milestone())]
        pub fn finalise_milestone_voting(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            milestone_key: MilestoneKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::do_finalise_milestone_voting(who, project_key, milestone_key)
        }

        /// Step 8 (INITATOR)
        /// Withdraw
        #[pallet::weight(<T as Config>::WeightInfo::withdraw())]
        pub fn withdraw(origin: OriginFor<T>, project_key: ProjectKey) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::new_withdrawal(who, project_key)
        }

        // Root Extrinsics:

        /// Set max proposal count per round
        #[pallet::weight(<T as Config>::WeightInfo::set_max_proposal_count_per_round(T::MaxProposalsPerRound::get()))]
        pub fn set_max_proposal_count_per_round(
            origin: OriginFor<T>,
            max_proposal_count_per_round: u32,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(
                max_proposal_count_per_round > 0
                    || max_proposal_count_per_round <= T::MaxProposalsPerRound::get(),
                Error::<T>::ParamLimitExceed
            );
            MaxProposalCountPerRound::<T>::put(max_proposal_count_per_round);

            Ok(().into())
        }


        /// Set milestone voting window
        #[pallet::weight(<T as Config>::WeightInfo::set_max_proposal_count_per_round(T::MaxProposalsPerRound::get()))]
        pub fn set_milestone_voting_window(
            origin: OriginFor<T>,
            new_milestone_voting_window: u32,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(
                new_milestone_voting_window > 0,
                Error::<T>::ParamLimitExceed
            );
            MilestoneVotingWindow::<T>::put(new_milestone_voting_window);

            Ok(().into())
        }



        /// Set withdrawal expiration
        #[pallet::weight(<T as Config>::WeightInfo::set_withdrawal_expiration())]
        pub fn set_withdrawal_expiration(
            origin: OriginFor<T>,
            withdrawal_expiration: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(
                withdrawal_expiration > (0_u32).into(),
                Error::<T>::InvalidParam
            );
            <WithdrawalExpiration<T>>::put(withdrawal_expiration);

            Ok(().into())
        }

        /// set is_identity_required
        #[pallet::weight(<T as Config>::WeightInfo::set_is_identity_required())]
        pub fn set_is_identity_required(
            origin: OriginFor<T>,
            is_identity_required: bool,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            IsIdentityRequired::<T>::put(is_identity_required);

            Ok(().into())
        }

        /// Ad Hoc Step (ADMIN)
        /// Refund
        #[pallet::weight(<T as Config>::WeightInfo::refund())]
        pub fn refund(origin: OriginFor<T>, project_key: ProjectKey) -> DispatchResultWithPostInfo {
            //ensure only admin can perform refund
            ensure_root(origin)?;
            Self::do_refund(project_key)
        }
    }
}

impl<T: Config> Pallet<T> {
    /// The account ID of the fund pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    pub fn ensure_initator(who: T::AccountId, project_key: ProjectKey) -> Result<(), Error<T>> {
        let project = Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        match project.initiator == who {
            true => Ok(()),
            false => Err(Error::<T>::UserIsNotInitator),
        }
    }

    pub fn project_account_id(key: ProjectKey) -> T::AccountId {
        T::PalletId::get().into_sub_account_truncating(key)
    }

    pub fn get_project(project_key: u32) -> Result<Project<AccountIdOf<T>, BalanceOf<T>, T::BlockNumber>, Error<T>> {
        Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)
    }

    pub fn get_total_project_contributions(project_key: u32) -> Result<BalanceOf<T>, Error<T>> {
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        // Calculate contribution amount
        let mut total_contribution_amount: BalanceOf<T> = (0_u32).into();
        for contribution in project.contributions.iter() {
            let contribution_value = contribution.value;
            total_contribution_amount += contribution_value;
        }
        Ok(total_contribution_amount)
    }

    fn new_project(
        who: T::AccountId,
        name: BoundedStringField,
        logo: BoundedStringField,
        description: BoundedDescriptionField,
        website: BoundedDescriptionField,
        proposed_milestones: BoundedProposedMilestones,
        required_funds: BalanceOf<T>,
        currency_id: common_types::CurrencyId,
    ) -> DispatchResultWithPostInfo {
        // Check if identity is required
        let is_identity_needed = IsIdentityRequired::<T>::get();
        if is_identity_needed {
            let identity = pallet_identity::Pallet::<T>::identity(who.clone())
                .ok_or(Error::<T>::IdentityNeeded)?;
            let mut is_found_judgement = false;
            for judgement in identity.judgements.iter() {
                if judgement.1 == pallet_identity::Judgement::Reasonable
                    || judgement.1 == pallet_identity::Judgement::KnownGood
                {
                    is_found_judgement = true;
                    break;
                }
            }
            ensure!(is_found_judgement, Error::<T>::IdentityNeeded);
        }

        // Validation
        ensure!(!name.is_empty(), Error::<T>::ProjectNameIsMandatory);
        ensure!(!logo.is_empty(), Error::<T>::LogoIsMandatory);
        ensure!(
            !description.is_empty(),
            Error::<T>::ProjectDescriptionIsMandatory
        );
        ensure!(!website.is_empty(), Error::<T>::WebsiteURLIsMandatory);

        let mut total_percentage = 0;
        for milestone in proposed_milestones.iter() {
            total_percentage += milestone.percentage_to_unlock;
        }
        ensure!(
            total_percentage == 100,
            Error::<T>::MilestonesTotalPercentageMustEqual100
        );

        let project_key = ProjectCount::<T>::get();
        let next_project_key = project_key.checked_add(1).ok_or(Error::<T>::Overflow)?;

        let mut milestones = Vec::new();
        let mut milestone_key: u32 = 0;

        // Fill in the proposals structure in advance
        for milestone in proposed_milestones {
            milestones.push(Milestone {
                project_key,
                milestone_key,
                name: milestone.name.to_vec(),
                percentage_to_unlock: milestone.percentage_to_unlock,
                is_approved: false,
            });
            milestone_key = milestone_key.checked_add(1).ok_or(Error::<T>::Overflow)?;
        }

        // Create a proposal
        let project = Project {
            name: name.clone().to_vec(),
            logo: logo.to_vec(),
            description: description.to_vec(),
            website: website.to_vec(),
            milestones,
            contributions: Vec::new(),
            required_funds,
            currency_id,
            withdrawn_funds: (0_u32).into(),
            initiator: who.clone(),
            create_block_number: <frame_system::Pallet<T>>::block_number(),
            approved_for_funding: false,
            funding_threshold_met: false,
            cancelled: false,
        };

        // Add proposal to list
        <Projects<T>>::insert(project_key, project);
        ProjectCount::<T>::put(next_project_key);

        Self::deposit_event(Event::ProjectCreated(
            who,
            name.to_vec(),
            project_key,
            required_funds,
            currency_id,
        ));

        Ok(().into())
    }

    fn new_round(
        start: T::BlockNumber,
        end: T::BlockNumber,
        project_keys: BoundedProjectKeys,
        round_type: RoundType
    ) -> DispatchResultWithPostInfo {
        let now = <frame_system::Pallet<T>>::block_number();
        // The number of items cannot exceed the maximum
        // ensure!(project_keyes.len() as u32 <= MaxProposalCountPerRound::<T>::get(), Error::<T>::ProposalAmountExceed);
        // The end block must be greater than the start block
        ensure!(end > start, Error::<T>::EndTooEarly);
        // Both the starting block number and the ending block number must be greater than the current number of blocks
        ensure!(end > now, Error::<T>::EndBlockNumberInvalid);

        // project_key should be smaller than project count
        let project_count = ProjectCount::<T>::get();

        // Ensure that the project keys will never be empty, this is done as it is an extrinsic parameter.
        ensure!(project_keys.len() > 0usize, Error::<T>::LengthMustExceedZero);
        let last_project = project_keys.last().expect("project keys length is validated; qed");

        ensure!(
            last_project < &project_count,
            Error::<T>::ProjectDoesNotExist
        );

        // Find the last valid round
        let key = RoundCount::<T>::get();

        let next_key = key.checked_add(1).ok_or(Error::<T>::Overflow)?;
        let round = RoundOf::<T>::new(
            start,
            end,
            project_keys.clone().into(),
            round_type.clone(),
        );

        //TODO: BUG:: key is used instead of next key. 
        // Add proposal round to list
        <Rounds<T>>::insert(key, Some(round));

        for project_key in project_keys.iter() {
            let project =
                Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

            //TODO: get explanation on this update. this will not work with vote of no confidence.
                // Update project withdrawn funds
            let updated_project = Project {
                name: project.name,
                logo: project.logo,
                description: project.description,
                website: project.website,
                milestones: project.milestones,
                contributions: project.contributions.clone(),
                required_funds: project.required_funds,
                currency_id: project.currency_id,
                withdrawn_funds: project.withdrawn_funds,
                initiator: project.initiator,
                create_block_number: project.create_block_number,
                approved_for_funding: true,
                funding_threshold_met: project.funding_threshold_met,
                cancelled: project.cancelled,
            };

            // Add proposal to list
            <Projects<T>>::insert(project_key, updated_project);
        }

        match round_type {
            RoundType::VotingRound => {Self::deposit_event(Event::VotingRoundCreated(key, project_keys.to_vec()))},
            RoundType::ContributionRound => {Self::deposit_event(Event::FundingRoundCreated(key, project_keys.to_vec()))},
            RoundType::VoteOfNoConfidence =>{Self::deposit_event(Event::NoConfidenceRoundCreated(key, project_keys.to_vec()))},
            }

        RoundCount::<T>::put(next_key);

        Ok(().into())
    }

    pub fn new_contribution(
        who: T::AccountId,
        project_key: ProjectKey,
        value: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        ensure!(value > (0_u32).into(), Error::<T>::InvalidParam);
        let project_count = ProjectCount::<T>::get();
        ensure!(project_key < project_count, Error::<T>::InvalidParam);
        let now = <frame_system::Pallet<T>>::block_number();

        // round list must be not none
        let round_key = RoundCount::<T>::get();
        ensure!(round_key > 0, Error::<T>::NoActiveRound);
        let mut project_exists_in_round = false;
        // Find processing round
        let mut processing_round: Option<RoundOf<T>> = None;
        for i in (0..round_key).rev() {

            let round = Self::rounds(i).ok_or(Error::<T>::KeyNotFound)?;
            if !round.is_canceled && round.start < now && round.end > now {
                // Find proposal by key
                for current_project_key in round.project_keys.iter() {
                    if current_project_key == &project_key {
                        project_exists_in_round = true;
                        processing_round = Some(round.clone());
                        break;
                    }
                }
            }
        }
        let round = processing_round.ok_or(Error::<T>::RoundNotProcessing)?;
        ensure!(project_exists_in_round, Error::<T>::ProjectNotInRound);
        let mut project =
            Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let mut max_cap = (0_u32).into();
        let mut new_contribution_value: BalanceOf<T> = value;
        let mut found_contribution: Option<&ContributionOf<T>> = None;
        let mut existing_contribution_index = 0;

        for (index, contribution) in project.contributions.iter().enumerate() {
            if contribution.account_id == who {
                new_contribution_value += contribution.value;
                found_contribution = Some(contribution);
                existing_contribution_index = index;
                break;
            }
        }

        // Find whitelist if exists
        if WhitelistSpots::<T>::contains_key(project_key) {
            let mut contributer_is_whitelisted = false;
            let whitelist_spots = Self::whitelist_spots(project_key).ok_or(Error::<T>::KeyNotFound)?;
            for whitelist_spot in whitelist_spots.clone().into_iter() {
                if whitelist_spot.who == who {
                    contributer_is_whitelisted = true;
                    max_cap = whitelist_spot.max_cap;
                    break;
                }
            }

            ensure!(
                contributer_is_whitelisted,
                Error::<T>::OnlyWhitelistedAccountsCanContribute
            );

            ensure!(
                max_cap == (0_u32).into() || max_cap >= new_contribution_value,
                Error::<T>::ContributionMustBeLowerThanMaxCap
            );
        }

        // Transfer contribute to proposal account
        T::MultiCurrency::transfer(
            project.currency_id,
            &who,
            &Self::project_account_id(project_key),
            value,
        )?;

        <Rounds<T>>::insert(round_key - 1, Some(round));

        Self::deposit_event(Event::ContributeSucceeded(
            who.clone(),
            project_key,
            value,
            project.currency_id,
            now,
        ));

        // Find previous contribution by account_id
        // If you have contributed before, then add to that contribution. Otherwise join the list.
        match found_contribution.clone() {
            Some(_contribution) => {
                // project.contributions.remove(&contribution);
                project.contributions.remove(existing_contribution_index);
                project.contributions.push(ContributionOf::<T> {
                    account_id: who.clone(),
                    value: new_contribution_value,
                });
            }
            None => {
                project.contributions.push(ContributionOf::<T> {
                    account_id: who.clone(),
                    value,
                });
            }
        }

        // Update project withdrawn funds
        let updated_project = Project {
            name: project.name,
            logo: project.logo,
            description: project.description,
            website: project.website,
            milestones: project.milestones,
            contributions: project.contributions.clone(),
            required_funds: project.required_funds,
            currency_id: project.currency_id,
            withdrawn_funds: project.withdrawn_funds,
            initiator: project.initiator,
            create_block_number: project.create_block_number,
            approved_for_funding: project.approved_for_funding,
            funding_threshold_met: project.funding_threshold_met,
            cancelled: project.cancelled,
        };

        // Add proposal to list
        <Projects<T>>::insert(project_key, updated_project);

        Ok(().into())
    }

    pub fn do_approve(
        project_key: ProjectKey,
        milestone_keys: Option<BoundedMilestoneKeys>,
    ) -> DispatchResultWithPostInfo {
        let round_key = RoundCount::<T>::get();
        // Find processing round
        let mut latest_round: Option<RoundOf<T>> = None;
        let mut project_exists_in_round = false;

        for i in (0..round_key).rev() {
            // Get the current round and check that both the key exists and the value under the key is some.
            let current_round = Self::rounds(i).ok_or(Error::<T>::KeyNotFound)?;

            if !current_round.is_canceled && current_round.project_keys.contains(&project_key) {
                latest_round = Some(current_round);
                project_exists_in_round = true;
                break;
            }
        }

        let round = latest_round.ok_or(Error::<T>::NoActiveRound)?;
        ensure!(!round.is_canceled, Error::<T>::RoundCanceled);

        // The round must have ended
        let now = <frame_system::Pallet<T>>::block_number();

        ensure!(project_exists_in_round, Error::<T>::ProjectNotInRound);

        let mut project =
            Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let total_contribution_amount: BalanceOf<T> =
            Self::get_total_project_contributions(project_key)?;


        let funds_matched = total_contribution_amount >= project.required_funds;
        if !funds_matched {
            // If the funds have not been matched then check if the round is over
            ensure!(round.end < now, Error::<T>::RoundNotEnded);

            // Once the round ends, check for the funding threshold met. (set threshold for 75%)
        }

        let mut milestones = project.milestones.clone();
        // set is_approved
        project.funding_threshold_met = true;
        if milestone_keys.is_some() {
            milestones = Vec::new();
            for mut milestone in project.milestones.into_iter() {
                for key in milestone_keys.as_ref().expect("is_some has been called; qed").iter() {
                    if &milestone.milestone_key == key {
                        let vote_lookup_key = (project_key, *key);
                        let votes_exist = MilestoneVotes::<T>::contains_key(vote_lookup_key);

                        let mut updated_vote = Vote {
                            yay: (0_u32).into(),
                            nay: (0_u32).into(),
                            is_approved: true,
                        };
                        milestone.is_approved = true;
                        if votes_exist {
                            let vote = <MilestoneVotes<T>>::get(vote_lookup_key).expect("milestone votes contains key has been called; qed");
                            updated_vote = Vote {
                                yay: vote.yay,
                                nay: vote.nay,
                                is_approved: true,
                            };
                        }

                        Self::deposit_event(Event::MilestoneApproved(project.initiator.clone(), project_key, *key, now));
                        <MilestoneVotes<T>>::insert(vote_lookup_key, updated_vote);
                    }
                }
                milestones.push(milestone.clone());
            }
        }
        <Rounds<T>>::insert(round_key, Some(round));

        // Update project milestones
        let updated_project = Project {
            name: project.name,
            logo: project.logo,
            description: project.description,
            website: project.website,
            milestones,
            contributions: project.contributions,
            required_funds: project.required_funds,
            currency_id: project.currency_id,
            withdrawn_funds: project.withdrawn_funds,
            initiator: project.initiator,
            create_block_number: project.create_block_number,
            approved_for_funding: project.approved_for_funding,
            funding_threshold_met: project.funding_threshold_met,
            cancelled: project.cancelled,
        };
        // Add proposal to list
        <Projects<T>>::insert(project_key, updated_project);
        Self::deposit_event(Event::ProjectApproved(round_key, project_key));
        Ok(().into())
    }

    // Take an approved project and submit an associated milestone.
    pub fn new_milestone_submission(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
    ) -> DispatchResultWithPostInfo {
        let now = <frame_system::Pallet<T>>::block_number();
        let project = Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        // Ensure that only the initiator has submitted and the project has been approved. 
        ensure!(project.initiator == who, Error::<T>::UserIsNotInitator);
        ensure!(
            project.funding_threshold_met,
            Error::<T>::OnlyApprovedProjectsCanSubmitMilestones
        );

        let end = now + MilestoneVotingWindow::<T>::get().into();
        let key = RoundCount::<T>::get();
        let round = RoundOf::<T>::new(now, end, vec![project_key], RoundType::VotingRound);
        let next_key = key.checked_add(1).ok_or(Error::<T>::Overflow)?;

        let vote = Vote {
            yay: (0_u32).into(),
            nay: (0_u32).into(),
            is_approved: false,
        };
        let vote_lookup_key = (project_key, milestone_key);
        <MilestoneVotes<T>>::insert(vote_lookup_key, vote);
        Self::deposit_event(Event::MilestoneSubmitted(who, project_key, milestone_key));
        // Add proposal round to list
        <Rounds<T>>::insert(key, Some(round));
        RoundCount::<T>::put(next_key);
        Self::deposit_event(Event::VotingRoundCreated(key, vec![project_key]));
        Ok(().into())
    }

    pub fn new_milestone_vote(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
        approve_milestone: bool,
    ) -> DispatchResultWithPostInfo {
        let project_count = ProjectCount::<T>::get();
        ensure!(project_key < project_count, Error::<T>::InvalidParam);
        let now = <frame_system::Pallet<T>>::block_number();

        // round list must be not none
        let round_key = RoundCount::<T>::get();
        ensure!(round_key > 0, Error::<T>::NoActiveRound);
        let project = Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        // Find processing round
        let mut latest_round: Option<RoundOf<T>> = None;
        let mut latest_round_key = 0;
        for i in (0..round_key).rev() {

            // TODO: expensive loop.
            let round = Self::rounds(i).ok_or(Error::<T>::KeyNotFound)?;
            if !round.is_canceled
                && round.start < now
                && round.end > now
                && round.project_keys.contains(&project_key)
            {
                latest_round = Some(round);
                latest_round_key = i;
                break;
            }
        }
        let round = latest_round.ok_or(Error::<T>::RoundNotProcessing)?;

        let mut existing_contributor = false;
        let mut contribution_amount: BalanceOf<T> = (0_u32).into();

        // Find previous contribution by account_id
        // If you have contributed before, then add to that contribution. Otherwise join the list.
        for contribution in project.contributions.iter() {
            if contribution.account_id == who {
                existing_contributor = true;
                contribution_amount = contribution.value;
                break;
            }
        }

        ensure!(existing_contributor, Error::<T>::OnlyContributorsCanVote);
        let vote_lookup_key = (who.clone(), project_key, milestone_key, latest_round_key);

        let vote_exists = UserVotes::<T>::contains_key(vote_lookup_key.clone());
        ensure!(!vote_exists, Error::<T>::VoteAlreadyExists);

        <UserVotes<T>>::insert(vote_lookup_key, approve_milestone);

        let user_milestone_vote = Self::milestone_votes((project_key, milestone_key)).ok_or(Error::<T>::KeyNotFound)?;

        if approve_milestone {
            let updated_vote = Vote {
                yay: user_milestone_vote.yay + contribution_amount,
                nay: user_milestone_vote.nay,
                is_approved: user_milestone_vote.is_approved,
            };
            <MilestoneVotes<T>>::insert((project_key, milestone_key), updated_vote)
        } else {
            let updated_vote = Vote {
                yay: user_milestone_vote.yay,
                nay: user_milestone_vote.nay + contribution_amount,
                is_approved: user_milestone_vote.is_approved,
            };
            <MilestoneVotes<T>>::insert((project_key, milestone_key), updated_vote)
        }

        <Rounds<T>>::insert(round_key - 1, Some(round));
        Self::deposit_event(Event::VoteComplete(
            who,
            project_key,
            milestone_key,
            approve_milestone,
            now,
        ));

        Ok(().into())
    }

    pub fn do_finalise_milestone_voting(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
    ) -> DispatchResultWithPostInfo {
        let project = Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        ensure!(
            project.initiator == who,
            Error::<T>::OnlyInitiatorOrAdminCanApproveMilestone
        );

        let total_contribution_amount: BalanceOf<T> =
            Self::get_total_project_contributions(project_key)?;

        let mut milestones = Vec::new();
        // set is_approved
        for mut milestone in project.milestones.into_iter() {
            if milestone.milestone_key == milestone_key {
                let vote_lookup_key = (project_key, milestone_key);
                let vote = Self::milestone_votes(vote_lookup_key).ok_or(Error::<T>::KeyNotFound)?;
                let total_votes = vote.yay + vote.nay;
                ensure!(
                    total_votes == total_contribution_amount,
                    Error::<T>::MilestoneVotingNotComplete
                );
                if vote.yay > vote.nay {
                    milestone.is_approved = true;
                    let updated_vote = Vote {
                        yay: vote.yay,
                        nay: vote.nay,
                        is_approved: true,
                    };
                    let now = <frame_system::Pallet<T>>::block_number();
                    Self::deposit_event(Event::MilestoneApproved(project.initiator.clone(), project_key, milestone_key, now));

                    <MilestoneVotes<T>>::insert(vote_lookup_key, updated_vote);
                }
            }
            milestones.push(milestone.clone());
        }

        // Update project milestones
        let updated_project = Project {
            name: project.name,
            logo: project.logo,
            description: project.description,
            website: project.website,
            milestones,
            contributions: project.contributions,
            required_funds: project.required_funds,
            currency_id: project.currency_id,
            withdrawn_funds: project.withdrawn_funds,
            initiator: project.initiator,
            create_block_number: project.create_block_number,
            approved_for_funding: project.approved_for_funding,
            funding_threshold_met: project.funding_threshold_met,
            cancelled: project.cancelled,
        };
        // Add proposal to list
        <Projects<T>>::insert(project_key, updated_project);

        Ok(().into())
    }

    pub fn new_withdrawal(who: T::AccountId, project_key: ProjectKey) -> DispatchResultWithPostInfo {
        let project = Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        ensure!(who == project.initiator, Error::<T>::InvalidAccount);
        let total_contribution_amount: BalanceOf<T> =
            Self::get_total_project_contributions(project_key)?;

        let mut unlocked_funds: BalanceOf<T> = (0_u32).into();
        for milestone in project.milestones.clone() {
            if milestone.is_approved {
                unlocked_funds += (total_contribution_amount
                    * milestone.percentage_to_unlock.into())
                    / 100u32.into();
            }
        }

        let available_funds: BalanceOf<T> = unlocked_funds - project.withdrawn_funds;
        ensure!(
            available_funds > (0_u32).into(),
            Error::<T>::NoAvailableFundsToWithdraw
        );

        T::MultiCurrency::transfer(
            project.currency_id,
            &Self::project_account_id(project_key),
            &project.initiator,
            available_funds,
        )?;

        // Update project withdrawn funds
        let updated_project = Project {
            name: project.name,
            logo: project.logo,
            description: project.description,
            website: project.website,
            milestones: project.milestones,
            contributions: project.contributions,
            required_funds: project.required_funds,
            currency_id: project.currency_id,
            withdrawn_funds: available_funds + project.withdrawn_funds,
            initiator: project.initiator,
            create_block_number: project.create_block_number,
            approved_for_funding: project.approved_for_funding,
            funding_threshold_met: project.funding_threshold_met,
            cancelled: project.cancelled,
        };
        // Add proposal to list
        <Projects<T>>::insert(project_key, updated_project);
        Self::deposit_event(Event::ProjectFundsWithdrawn(
            who,
            project_key,
            available_funds,
            project.currency_id,
        ));

        Ok(().into())
    }

    pub fn do_refund(project_key: ProjectKey) -> DispatchResultWithPostInfo {
        let project = Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        //getting the locked milestone percentage - these are also milestones that have not been approved
        let mut refunded_funds: BalanceOf<T> = 0_u32.into();
        let mut locked_milestone_percentage: u32 = 0;
        for milestone in project.milestones.clone() {
            if !milestone.is_approved {
                locked_milestone_percentage += milestone.percentage_to_unlock;
            }
        }

        for contribution in project.contributions.iter() {
            let who = contribution.account_id.clone();
            let refund_amount: BalanceOf<T> =
                (contribution.value * locked_milestone_percentage.into()) / 100u32.into();

            T::MultiCurrency::transfer(
                project.currency_id,
                &Self::project_account_id(project_key),
                &who,
                refund_amount,
            )?;

            refunded_funds += refund_amount;
        }

        // Update project cancellation status
        let updated_project = Project {
            name: project.name,
            logo: project.logo,
            description: project.description,
            website: project.website,
            milestones: project.milestones,
            contributions: project.contributions,
            required_funds: project.required_funds,
            currency_id: project.currency_id,
            withdrawn_funds: project.withdrawn_funds,
            initiator: project.initiator,
            create_block_number: project.create_block_number,
            approved_for_funding: project.approved_for_funding,
            funding_threshold_met: project.funding_threshold_met,
            cancelled: true,
        };
        // Updated new project status to chain
        <Projects<T>>::insert(project_key, updated_project);
        Self::deposit_event(Event::ProjectLockedFundsRefunded(
            project_key,
            refunded_funds,
        ));

        Ok(().into())
    }

    pub fn raise_no_confidence_round(who: T::AccountId, project_key: ProjectKey) -> DispatchResult {
        
        //ensure that who is a contributor or root
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        ensure!(project.contributions.iter().any(|c|{ c.account_id == who}), Error::<T>::InvalidAccount);


        //open a storage item for tracking the votes and who voted, use Vote struct.
        //
        Ok(().into())
    }
}

// The Constants associated with the bounded parameters
type MaxStringFieldLen = ConstU32<255>;
type MaxProjectKeys =  ConstU32<1000>; 
type MaxMileStoneKeys =  ConstU32<1000>; 
type MaxProposedMilestones = ConstU32<255>;
type MaxDescriptionField = ConstU32<5000>;
type MaxWhitelistPerProject = ConstU32<10000>;

pub type RoundKey = u32;
pub type ProjectKey = u32;
pub type MilestoneKey = u32;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
type ContributionOf<T> = Contribution<AccountIdOf<T>, BalanceOf<T>>;
type RoundOf<T> = Round<<T as frame_system::Config>::BlockNumber>;
// type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

// These are the bounded types which are suitable for handling user input due to their restriction of vector length.
type BoundedWhitelistSpots<T> = BoundedVec<Whitelist<AccountIdOf<T>, BalanceOf<T>>, MaxWhitelistPerProject>;
type BoundedProjectKeys = BoundedVec<ProjectKey, MaxProjectKeys>;
type BoundedMilestoneKeys = BoundedVec<ProjectKey, MaxMileStoneKeys>;
type BoundedStringField = BoundedVec<u8, MaxStringFieldLen>;
type BoundedProposedMilestones = BoundedVec<ProposedMilestone, MaxProposedMilestones>;
type BoundedDescriptionField = BoundedVec<u8, MaxDescriptionField>;

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub enum RoundType {
    ContributionRound,
    VotingRound,
    VoteOfNoConfidence,
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

// TODO: what is a proposal.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Proposal<AccountId, Balance, BlockNumber> {
    project_key: ProjectKey,
    contributions: Vec<Contribution<AccountId, Balance>>,
    is_approved: bool,
    is_canceled: bool,
    is_withdrawn: bool,
    withdrawal_expiration: BlockNumber,
}

/// The contribution users made to a proposal project.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Contribution<AccountId, Balance> {
    account_id: AccountId,
    value: Balance,
}

/// The contribution users made to a proposal project.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct ProposedMilestone {
    name: BoundedStringField,
    percentage_to_unlock: u32,
}

/// The contribution users made to a proposal project.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Milestone {
    project_key: ProjectKey,
    milestone_key: MilestoneKey,
    name: Vec<u8>,
    percentage_to_unlock: u32,
    is_approved: bool,
}

/// The vote struct is used to 
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Vote<Balance> {
    yay: Balance,
    nay: Balance,
    is_approved: bool,
}

/// The struct that holds the descriptive properties of a project.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Project<AccountId, Balance, BlockNumber> {
    name: Vec<u8>,
    logo: Vec<u8>,
    description: Vec<u8>,
    website: Vec<u8>,
    milestones: Vec<Milestone>,
    /// A collection of the accounts which have contributed and their contributions.
    contributions: Vec<Contribution<AccountId, Balance>>,
    currency_id: common_types::CurrencyId,
    required_funds: Balance,
    withdrawn_funds: Balance,
    /// The account that will receive the funds if the campaign is successful
    initiator: AccountId,
    create_block_number: BlockNumber,
    approved_for_funding: bool,
    funding_threshold_met: bool,
    cancelled: bool,
}

/// White struct
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Whitelist<AccountId, Balance> {
    who: AccountId,
    max_cap: Balance,
}
