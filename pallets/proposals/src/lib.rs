
//! Proposals Pallet
//! 
//! The Proposals pallet provides functionality for managing projects and milestones.
//!
//! ## Overview
//!
//! The Proposals pallet provides extrinsics for:
//! 
//! - Submitting a milestone. 
//! - Voting on a milestone's approval. 
//! - Withdrawing funds from a project.
//! - Initiating a dispute on a set of milestones.
//! - Refunding funds from a project.
//!
//! This also provides functionality for:
//! 
//! - Creating projects through the IntoProposal trait.
//! - An temporary multitoken system for minting foreign tokens to use on projects.
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, EncodeLike};
use common_types::CurrencyId;
use frame_support::{
    pallet_prelude::*, storage::bounded_btree_map::BoundedBTreeMap, traits::EnsureOrigin, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
pub use pallet::*;
use pallet_deposits::traits::DepositHandler;
use pallet_disputes::traits::DisputeRaiser;

use scale_info::TypeInfo;
use sp_arithmetic::per_things::Percent;
use sp_core::H256;
use sp_runtime::traits::{AccountIdConversion, One, Saturating, Zero};
use sp_std::{collections::btree_map::*, convert::TryInto, prelude::*};
use xcm::latest::MultiLocation;

pub mod traits;
use traits::{ExternalRefundHandler, IntoProposal};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod test_utils;

pub mod weights;
pub use weights::*;

pub mod migration;

pub mod impls;

pub use impls::*;
pub type ProjectKey = u32;
pub type MilestoneKey = u32;
pub type IndividualVotes<T> = BoundedBTreeMap<
    MilestoneKey,
    BoundedBTreeMap<AccountIdOf<T>, bool, <T as Config>::MaximumContributorsPerProject>,
    <T as Config>::MaxMilestonesPerProject,
>;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
pub type StorageItemOf<T> =
    <<T as Config>::DepositHandler as DepositHandler<BalanceOf<T>, AccountIdOf<T>>>::StorageItem;
pub type DepositIdOf<T> =
    <<T as Config>::DepositHandler as DepositHandler<BalanceOf<T>, AccountIdOf<T>>>::DepositId;
pub type MaxJuryOf<T> = <<T as Config>::JurySelector as pallet_fellowship::traits::SelectJury<
    AccountIdOf<T>,
>>::JurySize;

// These are the bounded types which are suitable for handling user input due to their restriction of vector length.
type BoundedBTreeMilestones<T> = BoundedBTreeMap<
    MilestoneKey,
    Milestone<BlockNumberFor<T>>,
    <T as Config>::MaxMilestonesPerProject,
>;
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
        /// The weights generated using the cli.
        type WeightInfo: WeightInfoT;
        /// The pallet_id used to generate sub accounts for each project fund pot.
        type PalletId: Get<PalletId>;
        /// The currency type.
        type MultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        /// Defines the length that a milestone can be voted on.
        type MilestoneVotingWindow: Get<BlockNumberFor<Self>>;
        /// The minimum percentage of votes, inclusive, that is required for a vote to pass.  
        type PercentRequiredForVoteToPass: Get<Percent>;
        /// Maximum number of contributors per project.
        type MaximumContributorsPerProject: Get<u32>;
        /// Maximum milestones allowed in a project.
        type MaxMilestonesPerProject: Get<u32>;
        /// Maximum project a user can submit, make sure its pretty big.
        type MaxProjectsPerAccount: Get<u32>;
        /// The maximum projects to be dealt with per block. Must be small as is dealt with in the hooks.
        type ExpiringProjectRoundsPerBlock: Get<u32>;
        /// Imbue fee in percent 0-99
        type ImbueFee: Get<Percent>;
        /// The account the imbue fee goes to.
        type ImbueFeeAccount: Get<AccountIdOf<Self>>;
        /// The type responisble for handling refunds.
        type ExternalRefundHandler: traits::ExternalRefundHandler<
            AccountIdOf<Self>,
            BalanceOf<Self>,
            CurrencyId,
        >;
        /// The type responsible for storage deposits.
        type DepositHandler: DepositHandler<BalanceOf<Self>, AccountIdOf<Self>>;
        /// The type that will be used to calculate the deposit of a project.
        type ProjectStorageItem: Get<StorageItemOf<Self>>;
        /// The trait that handler the raising of a dispute.
        type DisputeRaiser: DisputeRaiser<
            AccountIdOf<Self>,
            DisputeKey = ProjectKey,
            SpecificId = MilestoneKey,
            MaxSpecifics = Self::MaxMilestonesPerProject,
            MaxJurySize = MaxJuryOf<Self>,
        >;
        /// The jury selector type which is defining the max jury size.
        type JurySelector: pallet_fellowship::traits::SelectJury<AccountIdOf<Self>>;
        /// The origin responsible for setting the address responsible for minting tokens.
        type AssetSignerOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    /// Stores the projects of the pallet.
    #[pallet::storage]
    #[pallet::getter(fn projects)]
    pub type Projects<T: Config> = StorageMap<_, Identity, ProjectKey, Project<T>, OptionQuery>;

    /// The `AccountId` of the multichain signer
    #[pallet::storage]
    #[pallet::getter(fn key)]
    pub type ForeignCurrencySigner<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    // BTree of users that has voted, bounded by the number of contributors in a project.
    #[pallet::storage]
    pub(super) type UserHasVoted<T: Config> = StorageMap<
        _,
        Blake2_128,
        (ProjectKey, RoundType, MilestoneKey),
        BoundedBTreeMap<T::AccountId, bool, <T as Config>::MaximumContributorsPerProject>,
        ValueQuery,
    >;

    /// Stores the individuals votes on a given milestone key
    #[pallet::storage]
    pub type IndividualVoteStore<T: Config> =
        StorageMap<_, Blake2_128Concat, ProjectKey, ImmutableIndividualVotes<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn milestone_votes)]
    pub(super) type MilestoneVotes<T: Config> = StorageMap<
        _,
        Identity,
        ProjectKey,
        BoundedBTreeMap<MilestoneKey, Vote<BalanceOf<T>>, T::MaxMilestonesPerProject>,
        ValueQuery,
    >;

    /// Stores the completed project by a given initiator.
    #[pallet::storage]
    #[pallet::getter(fn completed_projects)]
    pub type CompletedProjects<T: Config> = StorageMap<
        _,
        Twox64Concat,
        AccountIdOf<T>,
        BoundedVec<ProjectKey, <T as Config>::MaxProjectsPerAccount>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn project_count)]
    pub type ProjectCount<T> = StorageValue<_, ProjectKey, ValueQuery>;

    /// Stores the ending block of the project key and round.
    #[pallet::storage]
    pub type Rounds<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (ProjectKey, MilestoneKey),
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

    // TODO: Check if this is in use.
    /// A helper to find what projects / milestones are in a dispute.
    #[pallet::storage]
    pub type ProjectsInDispute<T> = StorageMap<
        _,
        Blake2_128Concat,
        ProjectKey,
        BoundedVec<MilestoneKey, <T as Config>::MaxMilestonesPerProject>,
        ValueQuery,
    >;

    /// Projects in Voting round.
    /// A helper for the runtime api so we dont have to iterate over the Rounds Double map.
    #[pallet::storage]
    pub type ProjectInVoting<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProjectKey,
        Blake2_128Concat,
        MilestoneKey,
        (),
        ValueQuery,
    >;

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
        VoteSubmitted(
            T::AccountId,
            ProjectKey,
            MilestoneKey,
            bool,
            BlockNumberFor<T>,
        ),
        /// A milestone has been approved.
        MilestoneApproved(T::AccountId, ProjectKey, MilestoneKey, BlockNumberFor<T>),
        /// This milestone has been rejected.
        MilestoneRejected(ProjectKey, MilestoneKey),
        /// A project has been refunded either partially or completely.
        ProjectRefunded {
            project_key: ProjectKey,
            total_amount: BalanceOf<T>,
        },
        /// Foreign Asset Signer Changed
        ForeignAssetSignerChanged(T::AccountId),
        /// Foreign Asset Signer Changed
        ForeignAssetMinted(T::AccountId, T::AccountId, CurrencyId, BalanceOf<T>),
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
        /// Not enough funds in project account to distribute fees.
        NotEnoughFundsForFees,
        /// Conversion failed due to an error while funding the Project.
        ProjectFundingFailed,
        /// Conversion failed due to an error in milestone conversion (probably a bound has been abused).
        MilestoneConversionFailed,
        /// This project has too many refund locations.
        TooManyRefundLocations,
        /// This project has too many jury members.
        TooManyJuryMembers,
        /// There are too many milestone votes, this generally shouldnt be hit.
        TooManyMilestoneVotes,
        /// An internal error, a collection of votes for a milestone has been lost.s
        IndividualVoteNotFound,
        /// Only a contributor can raise a dispute.
        OnlyContributorsCanRaiseDispute,
        /// One of these milestones is already in a dispute.
        MilestonesAlreadyInDispute,
        /// You cannot raise a dispute on an approved milestone.
        CannotRaiseDisputeOnApprovedMilestone,
        /// Only a contributor can initiate a refund.
        OnlyContributorsCanInitiateRefund,
        /// Only the ForeignAssetSigner can mint tokens
        RequireForeignAssetSigner,
        /// A Jury is required to create a project.
        JuryRequired,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // SAFETY: ExpiringProjectRoundsPerBlock has to be sane to prevent overweight blocks.
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 1);
            let key_type_vec = RoundsExpiring::<T>::take(n);

            key_type_vec.iter().for_each(|item| {
                let (project_key, round_type, milestone_key) = item;
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

                // Remove the round prevents further voting.
                Rounds::<T>::remove((project_key, milestone_key), round_type);
                match round_type {
                    // Voting rounds automatically finalise if its reached its threshold.
                    // Therefore we can remove it on round end.
                    RoundType::VotingRound => {
                        weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));

                        MilestoneVotes::<T>::mutate(project_key, |vote_btree| {
                            vote_btree.remove(milestone_key);
                        });

                        IndividualVoteStore::<T>::mutate(project_key, |m_votes| {
                            if let Some(individual_votes) = m_votes {
                                individual_votes.clear_milestone_votes(*milestone_key);
                            }
                        });

                        ProjectInVoting::<T>::remove(project_key, milestone_key);
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

        /// Raise a dispute using the handle DisputeRaiser in the Config.
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::raise_dispute())]
        pub fn raise_dispute(
            origin: OriginFor<T>,
            project_key: ProjectKey,
            milestone_keys: BoundedVec<MilestoneKey, T::MaxMilestonesPerProject>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
            ensure!(
                milestone_keys
                    .iter()
                    .all(|ms_key| project.milestones.contains_key(ms_key)),
                Error::<T>::MilestoneDoesNotExist
            );
            ensure!(
                project.contributions.contains_key(&who),
                Error::<T>::OnlyContributorsCanRaiseDispute
            );
            ensure!(
                !ProjectsInDispute::<T>::contains_key(project_key),
                Error::<T>::MilestonesAlreadyInDispute
            );
            ensure!(
                !project.milestones.iter().any(|(milestone_key, milestone)| {
                    milestone_keys.contains(milestone_key) && milestone.is_approved
                }),
                Error::<T>::CannotRaiseDisputeOnApprovedMilestone
            );

            if project.jury.len() == 1 {
                // https://github.com/ImbueNetwork/imbue/issues/270
                let _ = <Self as pallet_disputes::traits::DisputeHooks<ProjectKey, MilestoneKey>>::on_dispute_complete(project_key, milestone_keys.to_vec(), pallet_disputes::DisputeResult::Success);
            } else {
                <T as Config>::DisputeRaiser::raise_dispute(
                    project_key,
                    who,
                    project.jury,
                    milestone_keys.clone(),
                )?;
                ProjectsInDispute::<T>::insert(project_key, milestone_keys);
            }

            Ok(())
        }

        /// Attempt a refund of milestones.
        /// Will only refund milestones that have can_refund set to true.
        #[pallet::call_index(15)]
        #[pallet::weight(<T as Config>::WeightInfo::refund())]
        pub fn refund(origin: OriginFor<T>, project_key: ProjectKey) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
            ensure!(
                project.contributions.contains_key(&who),
                Error::<T>::OnlyContributorsCanInitiateRefund
            );
            let project_account = Self::project_account_id(project_key);

            Projects::<T>::try_mutate_exists(project_key, |maybe_project| {
                if let Some(project) = maybe_project {
                    let mut total_to_refund_including_fee: BalanceOf<T> = Zero::zero();

                    for (_ms_key, ms) in project.milestones.iter_mut() {
                        if ms.can_refund && ms.transfer_status.is_none() {
                            let milestone_amount =
                                ms.percentage_to_unlock.mul_floor(project.raised_funds);
                            total_to_refund_including_fee =
                                total_to_refund_including_fee.saturating_add(milestone_amount);
                            ms.transfer_status = Some(TransferStatus::Refunded {
                                on: frame_system::Pallet::<T>::block_number(),
                            });
                        }
                    }

                    // Just so we dont multiply by zero.
                    ensure!(
                        total_to_refund_including_fee != Zero::zero(),
                        Error::<T>::NoAvailableFundsToWithdraw
                    );

                    let fee =
                        <T as Config>::ImbueFee::get().mul_floor(total_to_refund_including_fee);
                    // Take the fee and send to ImbueFeeAccount
                    T::MultiCurrency::transfer(
                        project.currency_id,
                        &project_account,
                        &<T as Config>::ImbueFeeAccount::get(),
                        fee,
                    )?;

                    let total_to_refund = total_to_refund_including_fee.saturating_sub(fee);

                    for (refund_location, percent_share) in &project.refund_locations {
                        let per_refund = percent_share.mul_floor(total_to_refund);
                        match refund_location {
                            Locality::Local(acc) => {
                                T::MultiCurrency::transfer(
                                    project.currency_id,
                                    &project_account,
                                    acc,
                                    per_refund,
                                )?;
                            }
                            Locality::Foreign(multilocation) => {
                                T::ExternalRefundHandler::send_refund_message_to_treasury(
                                    // TODO: change this to reference so that we dont have to clone....
                                    project_account.clone(),
                                    per_refund,
                                    project.currency_id,
                                    *multilocation,
                                )?;
                            }
                        }
                    }
                    project.refunded_funds = project
                        .refunded_funds
                        .saturating_add(total_to_refund_including_fee);
                    if project
                        .refunded_funds
                        .saturating_add(project.withdrawn_funds)
                        == project.raised_funds
                    {
                        *maybe_project = None;
                    }

                    Self::deposit_event(Event::<T>::ProjectRefunded {
                        project_key,
                        total_amount: total_to_refund_including_fee,
                    });
                    Ok::<(), DispatchError>(())
                } else {
                    Ok::<(), DispatchError>(())
                }
            })?;

            Ok(())
        }

        /// Sets the given AccountId (`new`) as the new Foreign asset signer
        /// key.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        #[pallet::call_index(16)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn set_foreign_asset_signer(
            origin: OriginFor<T>,
            new: AccountIdOf<T>,
        ) -> DispatchResult {
            T::AssetSignerOrigin::ensure_origin(origin)?;
            ForeignCurrencySigner::<T>::put(&new);
            Self::deposit_event(Event::ForeignAssetSignerChanged(new));
            Ok(())
        }

        /// Mints offchain assets to a users address
        ///
        /// The dispatch origin for this call must be the pre defined foreign asset signer.
        ///
        #[pallet::call_index(17)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn mint_offchain_assets(
            origin: OriginFor<T>,
            beneficiary: AccountIdOf<T>,
            currency_id: CurrencyId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::key().map_or(false, |authority_signer| who == authority_signer),
                Error::<T>::RequireForeignAssetSigner
            );
            <T as crate::Config>::MultiCurrency::deposit(currency_id, &beneficiary, amount)?;
            Self::deposit_event(Event::ForeignAssetMinted(
                who,
                beneficiary,
                currency_id,
                amount,
            ));

            Ok(())
        }
    }

    impl<T: crate::Config> IntoProposal<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>
        for crate::Pallet<T>
    where
        Project<T>: EncodeLike<Project<T>>,
    {
        type MaximumContributorsPerProject = T::MaximumContributorsPerProject;
        type MaxMilestonesPerProject = T::MaxMilestonesPerProject;
        type MaxJuryMembers = MaxJuryOf<T>;
        /// The caller is used to take the storage deposit from.
        /// With briefs and grants the caller is the beneficiary, so the fee will come from them.
        fn convert_to_proposal(
            currency_id: CurrencyId,
            contributions: ContributionsFor<T>,
            agreement_hash: H256,
            benificiary: AccountIdOf<T>,
            proposed_milestones: BoundedVec<ProposedMilestone, Self::MaxMilestonesPerProject>,
            refund_locations: BoundedVec<
                (Locality<AccountIdOf<T>>, Percent),
                Self::MaximumContributorsPerProject,
            >,
            jury: BoundedVec<AccountIdOf<T>, Self::MaxJuryMembers>,
            on_creation_funding: FundingPath,
            eoa: Option<common_types::ForeignOwnedAccount>,
        ) -> Result<(), DispatchError> {
            ensure!(jury.len() > 0, Error::<T>::JuryRequired);
            let project_key = crate::ProjectCount::<T>::get().saturating_add(1);

            // Take storage deposit only for a Project.
            let deposit_id = <T as Config>::DepositHandler::take_deposit(
                benificiary.clone(),
                <T as Config>::ProjectStorageItem::get(),
                CurrencyId::Native,
            )?;

            let project_account_id = crate::Pallet::<T>::project_account_id(project_key);
            // todo: Error handling here can be improved.
            let _is_funded = Self::fund_project(
                &on_creation_funding,
                &contributions,
                &project_account_id,
                currency_id,
            )
            .map_err(|_| Error::<T>::ProjectFundingFailed)?;

            let converted_milestones =
                Self::try_convert_to_milestones(proposed_milestones.clone(), project_key)
                    .map_err(|_| Error::<T>::MilestoneConversionFailed)?;
            let sum_of_contributions = contributions
                .values()
                .fold(Default::default(), |acc: BalanceOf<T>, x| {
                    acc.saturating_add(x.value)
                });

            let bounded_milestone_keys = proposed_milestones
                .iter()
                .enumerate()
                .map(|(i, _ms)| i as u32)
                .collect::<Vec<MilestoneKey>>()
                .try_into()
                .map_err(|_| Error::<T>::TooManyMilestones)?;

            let project: Project<T> = Project {
                agreement_hash,
                milestones: converted_milestones,
                contributions,
                currency_id,
                withdrawn_funds: Zero::zero(),
                raised_funds: sum_of_contributions,
                initiator: benificiary.clone(),
                created_on: frame_system::Pallet::<T>::block_number(),
                cancelled: false,
                deposit_id,
                refund_locations: refund_locations
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyRefundLocations)?,
                jury: jury
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyJuryMembers)?,
                on_creation_funding,
                refunded_funds: Zero::zero(),
                external_owned_address: eoa,
            };

            let individual_votes = ImmutableIndividualVotes::new(bounded_milestone_keys);
            IndividualVoteStore::<T>::insert(project_key, individual_votes);

            Projects::<T>::insert(project_key, project);
            ProjectCount::<T>::put(project_key);

            Self::deposit_event(Event::ProjectCreated(
                benificiary,
                agreement_hash,
                project_key,
                sum_of_contributions,
                currency_id,
                project_account_id,
            ));
            Ok(())
        }

        /// Convert a set of contributions into their respective refund locations.
        /// Only for local contributions.
        fn convert_contributions_to_refund_locations(
            contributions: &ContributionsFor<T>,
        ) -> BoundedVec<(Locality<AccountIdOf<T>>, Percent), T::MaximumContributorsPerProject>
        {
            let sum_of_contributions = contributions
                .values()
                .fold(Default::default(), |acc: BalanceOf<T>, x| {
                    acc.saturating_add(x.value)
                });

            let mut sum_of_percents: Percent = Zero::zero();
            let mut ret: BoundedVec<
                (Locality<AccountIdOf<T>>, Percent),
                T::MaximumContributorsPerProject,
            > = contributions
                .iter()
                .map(|c| {
                    let percent = Percent::from_rational(c.1.value, sum_of_contributions);
                    sum_of_percents = sum_of_percents.saturating_add(percent);
                    // Since these are local we can use MultiLocation::Default;
                    (Locality::from_local(c.0.clone()), percent)
                })
                .collect::<Vec<(Locality<AccountIdOf<T>>, Percent)>>()
                .try_into()
                .expect("Both input and output are bound by the same quantifier; qed");

            // TEST THIS
            if sum_of_percents != One::one() {
                // We are missing a part of the fund so take the remainder and use the pallet_id as the return address.
                //(as is used throughout the rest of the pallet for fees)
                let diff = <Percent as One>::one().saturating_sub(sum_of_percents);
                // TODO: IF THE CONTRIBUTION BOUND IS MAX ALREADY THEN WE CANNOT PUSH THE DUST ACCOUNT ON
                // FAIL SILENTLY AND CLEAN UP ON FINAL WITHDRAW INSTEAD.
                let _ = ret.try_push((
                    Locality::from_local(<T as Config>::ImbueFeeAccount::get()),
                    diff,
                ));
            }

            ret
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Copy, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub enum RoundType {
    VotingRound,
    VoteOfNoConfidence,
}

/// The milestones provided by the user to define the milestones of a project.
/// TODO: add ipfs hash like in the grants pallet and
/// TODO: move these to a common repo (common_types will do)
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct ProposedMilestone {
    pub percentage_to_unlock: Percent,
}

/// The contribution users made to a project project.
/// TODO: move these to a common repo (common_types will do)
/// TODO: add ipfs hash like in the grants pallet and

// TODO: MIGRATION FOR MILESTONES
//can_refund
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct Milestone<BlockNumber> {
    pub project_key: ProjectKey,
    pub milestone_key: MilestoneKey,
    pub percentage_to_unlock: Percent,
    pub is_approved: bool,
    pub can_refund: bool,
    pub transfer_status: Option<TransferStatus<BlockNumber>>,
}

impl<B> Milestone<B> {
    fn new(
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
        percentage_to_unlock: Percent,
    ) -> Self {
        Self {
            project_key,
            milestone_key,
            percentage_to_unlock,
            is_approved: false,
            can_refund: false,
            transfer_status: None,
        }
    }
}

/// The vote struct is used to
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct Vote<Balance> {
    yay: Balance,
    nay: Balance,
    is_approved: bool,
}

impl<Balance: Zero> Default for Vote<Balance> {
    fn default() -> Self {
        Self {
            yay: Zero::zero(),
            nay: Zero::zero(),
            is_approved: false,
        }
    }
}

// TODO MILESTONE MIGRATIONS
/// The struct which contain milestones that can be submitted.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Project<T: Config> {
    pub agreement_hash: H256,
    pub milestones: BoundedBTreeMilestones<T>,
    /// The contributions to a project, also known as milestone approvers. TODO: discuss name change.
    pub contributions: ContributionsFor<T>,
    /// The currency id of the Project's funds.
    pub currency_id: common_types::CurrencyId,
    /// The amount of funds already withdrawn from the project.
    pub withdrawn_funds: BalanceOf<T>,
    /// The amount of money actually raised on instantiation of the Project.
    pub raised_funds: BalanceOf<T>,
    /// The initiator of the project, also known as the beneficiary: TODO: discuss name change.
    pub initiator: AccountIdOf<T>,
    /// The blocknumber the Project was created on
    pub created_on: BlockNumberFor<T>,
    /// is the project cancelled TODO: make an issue this is from legacy.
    pub cancelled: bool,
    /// The deposit_id is reponsible for returning deposits held in pallet-deposits.
    pub deposit_id: DepositIdOf<T>,
    /// Where do the refunds end up and what percent they get.
    pub refund_locations:
        BoundedVec<(Locality<AccountIdOf<T>>, Percent), T::MaximumContributorsPerProject>,
    /// Who should deal with disputes.
    pub jury: BoundedVec<AccountIdOf<T>, MaxJuryOf<T>>,
    /// When is the project funded and how is it taken.
    pub on_creation_funding: FundingPath,
    /// The amount of funds refunded.
    pub refunded_funds: BalanceOf<T>,
    /// The payment address used when the currency_id is of type foreign.
    pub external_owned_address: Option<common_types::ForeignOwnedAccount>,
}

/// For deriving the location of an account.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub enum Locality<AccountId> {
    Local(AccountId),
    Foreign(MultiLocation),
}

impl<AccountId> Locality<AccountId> {
    fn _from_multilocation(m: MultiLocation) -> Self {
        Self::Foreign(m)
    }
    fn from_local(l: AccountId) -> Self {
        Self::Local(l)
    }
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

/// Defines how a project is funded on its instantiation.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen, Default)]
pub enum FundingPath {
    // TODO: Possibly wise to change this to actually define where the reserves are coming from.
    // This allows us to break the notion of a "contributor" finally and worry only about the "approvers".
    /// Take from the reserved amounts of the contributors account.
    #[default]
    TakeFromReserved,
    /// Take nothing from the contributors and await funding from some outside source.
    WaitForFunding,
}

/// Defines how the funds were taken out of a specific milestone.
/// Contians the block number for possible further investigation.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub enum TransferStatus<BlockNumber> {
    Refunded { on: BlockNumber },
    Withdrawn { on: BlockNumber },
}

/// Stores the btree for each individual vote.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ImmutableIndividualVotes<T: Config> {
    votes: IndividualVotes<T>,
}

pub trait WeightInfoT {
    fn submit_milestone() -> Weight;
    fn vote_on_milestone() -> Weight;
    fn withdraw() -> Weight;
    fn on_initialize() -> Weight;
    fn raise_dispute() -> Weight;
    fn refund() -> Weight;
}
