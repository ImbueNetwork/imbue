use crate::*;
use common_types::milestone_origin::FundingType;
use pallet_identity::Judgement;
use sp_runtime::traits::Saturating;
use sp_std::{collections::btree_map::BTreeMap, vec};
pub const MAX_PERCENTAGE: u32 = 100u32;
use scale_info::prelude::format;

/// <HB SBP Review:
///
/// I would use checked_div for some divisions to be sure.
///
/// >

impl<T: Config> Pallet<T> {
    /// The account ID of the fund pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    pub fn ensure_initiator(who: T::AccountId, project_key: ProjectKey) -> Result<(), Error<T>> {
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        match project.initiator == who {
            true => Ok(()),
            false => Err(Error::<T>::UserIsNotInitiator),
        }
    }

    pub fn project_account_id(key: ProjectKey) -> ProjectAccountId<T> {
        T::PalletId::get().into_sub_account_truncating(format!("//{key}"))
    }

    pub fn new_project(
        who: T::AccountId,
        agreement_hash: H256,
        proposed_milestones: BoundedProposedMilestones<T>,
        required_funds: BalanceOf<T>,
        currency_id: common_types::CurrencyId,
        funding_type: FundingType,
    ) -> Result<ProjectKey, DispatchError> {
        // Check if identity is required
        if <T as Config>::IsIdentityRequired::get() {
            Self::ensure_identity_is_decent(&who)?;
        }

        <T as Config>::MultiCurrency::reserve(
            CurrencyId::Native,
            &who,
            T::ProjectStorageDeposit::get(),
        )
        .map_err(|_| Error::<T>::ImbueRequiredForStorageDep)?;

        let project_key = ProjectCount::<T>::get();
        let next_project_key = project_key.checked_add(1).ok_or(Error::<T>::Overflow)?;

        let mut milestone_key: u32 = 0;

        let mut milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();

        // Fill in the projects structure in advance
        for milestone in proposed_milestones {
            let milestone = Milestone {
                project_key,
                milestone_key,
                percentage_to_unlock: milestone.percentage_to_unlock,
                is_approved: false,
            };
            milestones.insert(milestone_key, milestone);
            milestone_key = milestone_key.checked_add(1).ok_or(Error::<T>::Overflow)?;
        }

        // Create a project
        let project = Project {
            agreement_hash,
            milestones,
            contributions: BTreeMap::new(),
            required_funds,
            currency_id,
            raised_funds: (0_u32).into(),
            withdrawn_funds: (0_u32).into(),
            initiator: who.clone(),
            created_on: <frame_system::Pallet<T>>::block_number(),
            approved_for_funding: false,
            funding_threshold_met: false,
            cancelled: false,
            funding_type,
        };

        // Add project to list
        <Projects<T>>::insert(project_key, project);
        ProjectCount::<T>::put(next_project_key);
        let project_account = Self::project_account_id(project_key);
        Self::deposit_event(Event::ProjectCreated(
            who,
            agreement_hash,
            project_key,
            required_funds,
            currency_id,
            project_account,
        ));

        Ok(project_key)
    }

    pub fn try_update_existing_project(
        who: T::AccountId,
        project_key: ProjectKey,
        proposed_milestones: BoundedProposedMilestones<T>,
        required_funds: BalanceOf<T>,
        currency_id: CurrencyId,
        agreement_hash: H256,
    ) -> DispatchResultWithPostInfo {
        // Check if identity is required
        if <T as Config>::IsIdentityRequired::get() {
            Self::ensure_identity_is_decent(&who)?;
        }

        //check to ensure valid and existing project
        let mut project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        ensure!(project.initiator == who, Error::<T>::UserIsNotInitiator);

        ensure!(
            !project.approved_for_funding,
            Error::<T>::ProjectAlreadyApproved
        );

        let mut milestone_key: u32 = 0;

        let mut milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();

        // Fill in the projects structure in advance
        for milestone in proposed_milestones {
            let milestone = Milestone {
                project_key,
                milestone_key,
                percentage_to_unlock: milestone.percentage_to_unlock,
                is_approved: false,
            };
            milestones.insert(milestone_key, milestone.clone());
            milestone_key = milestone_key.checked_add(1).ok_or(Error::<T>::Overflow)?;
        }

        // Update project
        project.milestones = milestones;
        project.required_funds = required_funds;
        project.currency_id = currency_id;
        project.agreement_hash = agreement_hash;

        /// <HB SBP Review:
        ///
        /// Maybe instead of using inset, this is a good candidate for try_mutate as well?
        ///
        /// >
        // Add project to list
        <Projects<T>>::insert(project_key, project);

        Ok(().into())
    }

    pub fn new_round(
        start: T::BlockNumber,
        end: T::BlockNumber,
        project_keys: BoundedProjectKeys,
        round_type: RoundType,
    ) -> DispatchResultWithPostInfo {
        // Find the last valid round
        let round_key = RoundCount::<T>::get()
            .checked_add(1)
            .ok_or(Error::<T>::Overflow)?;
        RoundCount::<T>::put(round_key);

        let round = RoundOf::<T>::new(start, end, project_keys.clone().into(), round_type.clone());

        // Add project round to list
        <Rounds<T>>::insert(round_key, Some(round));

        // Project keys is bounded to 5 projects maximum.
        for project_key in project_keys.iter() {
            //Try update project as approved for funding, assuming only RoundType::Contribution will be used.
            Projects::<T>::try_mutate(project_key, |project| -> DispatchResult {
                if let Some(p) = project {
                    p.approved_for_funding = true
                }
                Ok(())
            })?;
        }

        match round_type {
            RoundType::VotingRound => {
                Self::deposit_event(Event::VotingRoundCreated(round_key, project_keys.to_vec()))
            }
            RoundType::ContributionRound => {
                Self::deposit_event(Event::FundingRoundCreated(round_key, project_keys.to_vec()))
            }
            _ => {}
        }

        Ok(().into())
    }

    pub fn new_contribution(
        who: T::AccountId,
        round_key: RoundKey,
        project_key: ProjectKey,
        value: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        /// <HB SBP Review:
        ///
        /// I would avoid hardocoding types of this kind. Please use Zero::zero() instead.
        ///
        /// >
        // TODO add configurable value for min and max contribution per contributor
        ensure!(value > (0_u32).into(), Error::<T>::InvalidParam);
        let now = <frame_system::Pallet<T>>::block_number();

        // round list must be not none
        let round = Self::rounds(round_key).ok_or(Error::<T>::KeyNotFound)?;

        ensure!(
            round.round_type == RoundType::ContributionRound,
            Error::<T>::InvalidRoundType
        );

        ensure!(round.start <= now, Error::<T>::StartBlockNumberInvalid);

        ensure!(round.end >= now, Error::<T>::EndBlockNumberInvalid);

        ensure!(
            round.project_keys.contains(&project_key),
            Error::<T>::ProjectNotInRound
        );

        let mut project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        let new_amount = match project.contributions.get(&who) {
            Some(contribution) => contribution.value,
            None => BalanceOf::<T>::default(),
        }
        .saturating_add(value);

        // Find whitelist if exists
        if WhitelistSpots::<T>::contains_key(project_key) {
            let whitelist_spots =
                Self::whitelist_spots(project_key).ok_or(Error::<T>::WhiteListNotFound)?;
            ensure!(
                whitelist_spots.contains_key(&who),
                Error::<T>::OnlyWhitelistedAccountsCanContribute
            );

            let default_max_cap: BalanceOf<T> = (0u32).into();
            let max_cap = *whitelist_spots.get(&who).unwrap_or(&default_max_cap);

            ensure!(
                max_cap == default_max_cap || max_cap >= new_amount,
                Error::<T>::ContributionMustBeLowerThanMaxCap
            );
        }

        // Transfer contribute to project account
        T::MultiCurrency::transfer(
            project.currency_id,
            &who,
            &Self::project_account_id(project_key),
            value,
        )?;

        Self::deposit_event(Event::ContributeSucceeded(
            who.clone(),
            project_key,
            value,
            project.currency_id,
            now,
        ));

        project.contributions.insert(
            who,
            Contribution {
                value: new_amount,
                timestamp: frame_system::Pallet::<T>::block_number(),
            },
        );
        project.raised_funds = project.raised_funds.saturating_add(value);

        // Update storage item to include the new contributions.
        <Projects<T>>::insert(project_key, project.clone());

        Ok(().into())
    }

    pub fn do_approve(
        project_key: ProjectKey,
        round_key: RoundKey,
        milestone_keys: Option<BoundedMilestoneKeys<T>>,
    ) -> DispatchResultWithPostInfo {
        let round = Self::rounds(round_key).ok_or(Error::<T>::KeyNotFound)?;
        ensure!(
            round.project_keys.contains(&project_key),
            Error::<T>::ProjectNotInRound
        );
        ensure!(!round.is_canceled, Error::<T>::RoundCanceled);
        let now = <frame_system::Pallet<T>>::block_number();
        let mut project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let total_contribution_amount: BalanceOf<T> = project.raised_funds;

        let funds_matched = total_contribution_amount >= project.required_funds;
        if !funds_matched {
            // If the funds have not been matched then check if the round is over
            ensure!(round.end < now, Error::<T>::RoundNotEnded);
            // TODO: PR for this exists.
            // Once the round ends, check for the funding threshold met. (set threshold for 75%)
        }
        project.funding_threshold_met = true;
        // Warning: This will allow the withdrawal of funds, approve is a governance action so should not be a problem.
        // Consider removing this/
        if let Some(ms_keys) = milestone_keys {
            for milestone_key in ms_keys.into_iter() {
                ensure!(
                    project.milestones.contains_key(&milestone_key),
                    Error::<T>::MilestoneDoesNotExist
                );

                let vote_lookup_key = (project_key, milestone_key);

                MilestoneVotes::<T>::try_mutate(vote_lookup_key, |maybe_vote| {
                    if let Some(vote) = maybe_vote {
                        vote.is_approved = true;
                    } else {
                        *maybe_vote = Some(Vote::default())
                    }

                    Ok::<(), Error<T>>(())
                })?;

                Self::deposit_event(Event::MilestoneApproved(
                    project.initiator.clone(),
                    project_key,
                    milestone_key,
                    now,
                ));

                let mut milestone = project
                    .milestones
                    .get_mut(&milestone_key)
                    .ok_or(Error::<T>::MilestoneDoesNotExist)?;
                milestone.is_approved = true;
            }
        }
        <Rounds<T>>::insert(round_key, Some(round));
        <Projects<T>>::insert(project_key, project);
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
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        // Ensure that only the initiator has submitted and the project has been approved.
        ensure!(project.initiator == who, Error::<T>::UserIsNotInitiator);
        ensure!(
            project.funding_threshold_met,
            Error::<T>::OnlyApprovedProjectsCanSubmitMilestones
        );

        let end = now + <T as Config>::MilestoneVotingWindow::get();

        let round_key = RoundCount::<T>::get()
            .checked_add(1)
            .ok_or(Error::<T>::Overflow)?;

        let round = RoundOf::<T>::new(now, end, vec![project_key], RoundType::VotingRound);

        let vote = Vote::default();
        let vote_lookup_key = (project_key, milestone_key);
        <MilestoneVotes<T>>::insert(vote_lookup_key, vote);
        Self::deposit_event(Event::MilestoneSubmitted(who, project_key, milestone_key));
        // Add project round to list
        <Rounds<T>>::insert(round_key, Some(round));
        RoundCount::<T>::put(round_key);
        Self::deposit_event(Event::VotingRoundCreated(round_key, vec![project_key]));
        Ok(().into())
    }

    pub fn new_milestone_vote(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
        round_key: RoundKey,
        approve_milestone: bool,
    ) -> DispatchResultWithPostInfo {
        let project_count = ProjectCount::<T>::get();
        ensure!(project_key < project_count, Error::<T>::InvalidParam);
        let now = <frame_system::Pallet<T>>::block_number();

        // round list must be not none
        let mut project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let round = Self::rounds(round_key).ok_or(Error::<T>::KeyNotFound)?;
        ensure!(
            round.round_type == RoundType::VotingRound,
            Error::<T>::InvalidRoundType
        );

        ensure!(round.start < now, Error::<T>::StartBlockNumberInvalid);

        ensure!(round.end > now, Error::<T>::EndBlockNumberInvalid);

        ensure!(
            round.project_keys.contains(&project_key),
            Error::<T>::ProjectNotInRound
        );

        ensure!(
            project.contributions.contains_key(&who),
            Error::<T>::OnlyContributorsCanVote
        );
        let contribution_amount = Self::ensure_contributor_of(&project, &who)?;
        let vote_lookup_key = (who.clone(), project_key, milestone_key, round_key);

        let vote_exists = UserVotes::<T>::contains_key(&vote_lookup_key);
        ensure!(!vote_exists, Error::<T>::VoteAlreadyExists);

        <UserVotes<T>>::insert(vote_lookup_key, approve_milestone);

        let existing_milestone_vote =
            Self::milestone_votes((project_key, milestone_key)).ok_or(Error::<T>::KeyNotFound)?;
        let mut updated_vote = Vote::default();
        if approve_milestone {
            updated_vote = Vote {
                yay: existing_milestone_vote
                    .yay
                    .saturating_add(contribution_amount),
                nay: existing_milestone_vote.nay,
                is_approved: existing_milestone_vote.is_approved,
            };
            Self::deposit_event(Event::VoteComplete(
                who,
                project_key,
                milestone_key,
                approve_milestone,
                now,
            ));
            //once the voting is complete check if the milestone is eligible for auto approval
            //Getting the total threshold required for the milestone to be approved based on the raised funds
            let funding_threshold: BalanceOf<T> = project
                .raised_funds
                .saturating_mul(T::PercentRequiredForVoteToPass::get().into())
                / 100u32.into();

            let mut milestone = project
                .milestones
                .get_mut(&milestone_key)
                .ok_or(Error::<T>::KeyNotFound)?;
            //if the yay votes are both greater than the nay votes and the funding threshold then the milestone is approved
            if updated_vote.yay >= funding_threshold {
                milestone.is_approved = true;
                updated_vote = Vote {
                    yay: updated_vote.yay,
                    nay: updated_vote.nay,
                    is_approved: true,
                };
                let now = <frame_system::Pallet<T>>::block_number();
                Self::deposit_event(Event::MilestoneApproved(
                    project.initiator.clone(),
                    project_key,
                    milestone_key,
                    now,
                ));
                <Projects<T>>::insert(project_key, &project);
            }
        } else {
            updated_vote = Vote {
                yay: existing_milestone_vote.yay,
                nay: existing_milestone_vote
                    .nay
                    .saturating_add(contribution_amount),
                is_approved: existing_milestone_vote.is_approved,
            };
            Self::deposit_event(Event::VoteComplete(
                who,
                project_key,
                milestone_key,
                approve_milestone,
                now,
            ));
        }
        <Rounds<T>>::insert(round_key, Some(round));
        <MilestoneVotes<T>>::insert((project_key, milestone_key), &updated_vote);

        Ok(().into())
    }

    pub fn do_finalise_milestone_voting(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
    ) -> DispatchResultWithPostInfo {
        let mut project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        ensure!(
            project.initiator == who,
            Error::<T>::OnlyInitiatorOrAdminCanApproveMilestone
        );

        ensure!(
            project.milestones.contains_key(&milestone_key),
            Error::<T>::MilestoneDoesNotExist
        );

        /// <HB SBP Review:
        ///
        /// Please remove this unwrap and manage the error properly.
        ///
        /// >
        let mut milestone = project.milestones.get_mut(&milestone_key).unwrap().clone();

        // set is_approved
        let vote_lookup_key = (project_key, milestone_key);
        let vote = Self::milestone_votes(vote_lookup_key).ok_or(Error::<T>::KeyNotFound)?;

        // let the 100 x threshold required = total_votes * majority required
        let threshold_votes: BalanceOf<T> = project
            .raised_funds
            .saturating_mul(T::PercentRequiredForVoteToPass::get().into());
        let percent_multiple: BalanceOf<T> = 100u32.into();

        ensure!(
            percent_multiple.saturating_mul(vote.yay + vote.nay) >= threshold_votes,
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
            Self::deposit_event(Event::MilestoneApproved(
                project.initiator.clone(),
                project_key,
                milestone_key,
                now,
            ));
            <MilestoneVotes<T>>::insert(vote_lookup_key, updated_vote);
        }
        //TODO: Case for equal votes?

        project.milestones.insert(milestone_key, milestone);
        <Projects<T>>::insert(project_key, project);

        Ok(().into())
    }

    pub fn new_withdrawal(
        who: T::AccountId,
        project_key: ProjectKey,
    ) -> DispatchResultWithPostInfo {
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        ensure!(!project.cancelled, Error::<T>::ProjectWithdrawn);
        ensure!(who == project.initiator, Error::<T>::InvalidAccount);
        //ensure that the project is approved_for_funding?

        let unlocked_funds: BalanceOf<T> =
            project
                .milestones
                .iter()
                .fold(Default::default(), |acc, ms| {
                    if ms.1.is_approved {
                        let per_milestone = project
                            .raised_funds
                            .saturating_mul(ms.1.percentage_to_unlock.into())
                            / MAX_PERCENTAGE.into();
                        acc.saturating_add(per_milestone)
                    } else {
                        acc
                    }
                });

        let withdrawable: BalanceOf<T> = unlocked_funds.saturating_sub(project.withdrawn_funds);

        ensure!(
            withdrawable > (0_u32).into(),
            Error::<T>::NoAvailableFundsToWithdraw
        );

        /// HB SBP Review:
        ///
        /// This is a good example about how sp_arithmetic can be used to manage percentages in a safe way.
        ///
        /// >
        let fee = withdrawable.saturating_mul(<T as Config>::ImbueFee::get().into())
            / MAX_PERCENTAGE.into();
        let withdrawn = withdrawable.saturating_sub(fee);

        let project_account = Self::project_account_id(project_key);
        let pallet_account = Self::account_id();

        // Take the fee
        T::MultiCurrency::transfer(project.currency_id, &project_account, &pallet_account, fee)?;

        T::MultiCurrency::transfer(
            project.currency_id,
            &project_account,
            &project.initiator,
            withdrawn,
        )?;

        Projects::<T>::mutate_exists(project_key, |project| -> DispatchResult {
            if let Some(p) = project {
                p.withdrawn_funds = p.withdrawn_funds.saturating_add(withdrawable);
                if p.withdrawn_funds == p.raised_funds {
                    Self::reinstate_storage_deposit(&p.initiator)?;
                    *project = None;
                }
            }
            Ok(())
        })?;

        Self::deposit_event(Event::ProjectFundsWithdrawn(
            who,
            project_key,
            withdrawn,
            project.currency_id,
        ));

        Ok(().into())
    }

    /// Appends a list of refunds to the queue to be used by the hooks.
    // DEPRICATED, PLS REMOOVE.
    pub fn add_refunds_to_queue_depricated(project_key: ProjectKey) -> DispatchResultWithPostInfo {
        let mut project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        //getting the locked milestone percentage - these are also milestones that have not been approved
        let mut refunded_funds: BalanceOf<T> = 0_u32.into();
        let mut locked_milestone_percentage: u32 = 0;
        for (_milestone_key, milestone) in project.milestones.clone() {
            if !milestone.is_approved {
                locked_milestone_percentage =
                    locked_milestone_percentage.saturating_add(milestone.percentage_to_unlock);
            }
        }

        let mut current_refunds = RefundQueue::<T>::get();

        // TODO: How can we refund all contributions without looping?
        for (who, contribution) in project.contributions.iter() {
            let project_account_id = Self::project_account_id(project_key);

            /// <HB SBP Review:
            ///
            /// Unsafe operation. Please use checked_mul or saturated_mul instead.
            /// And for divisions i always try to use cheched_div just to be sure.
            /// >
            let refund_amount: BalanceOf<T> =
                ((contribution).value * locked_milestone_percentage.into()) / MAX_PERCENTAGE.into();

            current_refunds.push((
                who.clone(),
                project_account_id.clone(),
                refund_amount,
                project.currency_id,
            ));
            //
            refunded_funds = refunded_funds.saturating_add(refund_amount);
        }

        // Updated new project status to cancelled
        project.cancelled = true;
        <Projects<T>>::insert(project_key, project);
        RefundQueue::<T>::put(current_refunds);

        Self::deposit_event(Event::ProjectFundsAddedToRefundQueue(
            project_key,
            refunded_funds,
        ));
        Ok(().into())
    }

    /// Using the parameters provided (which should be from the refund queue),
    /// Process a refund.
    /// Used in hooks so cannot panic.
    /// DEPRICATED
    pub fn refund_item_in_queue_depricated(
        from: &T::AccountId,
        to: &T::AccountId,
        amount: BalanceOf<T>,
        currency_id: CurrencyId,
    ) -> bool {
        let can_withraw: DispatchResult =
            T::MultiCurrency::ensure_can_withdraw(currency_id, from, amount);
        if can_withraw.is_ok() {
            // this should pass now, but i will not return early
            let _ = T::MultiCurrency::transfer(currency_id, from, to, amount);
            true
        } else {
            false
        }
    }

    /// Split off an amount of refunds off the vector and place into refund storage.
    /// Returns a boolean if a split off has succeeded.
    /// Used in hooks so cannot panic.
    /// DEPRICATED
    pub fn split_off_refunds_depricated(refunds: &mut Refunds<T>, c: u32) -> bool {
        // split_off panics when at > len:
        // https://paritytech.github.io/substrate/master/sp_std/vec/struct.Vec.html#method.split_off
        // If the length is zero do nothing
        if c == 0 {
            return false;
        }

        if c as usize <= refunds.len() {
            // If its a legitimate operation, split off.
            RefundQueue::<T>::put(refunds.split_off(c as usize));
            true
        } else {
            // panic case we will place in an empty vec as the counter is wrong.
            RefundQueue::<T>::kill();
            false
        }
    }

    /// This function raises a vote of no confidence.
    /// This round can only be called once and there after can only be voted on.
    /// The person calling it must be a contributor.
    pub fn raise_no_confidence_round(who: T::AccountId, project_key: ProjectKey) -> DispatchResult {
        //ensure that who is a contributor or root
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let contribution = Self::ensure_contributor_of(&project, &who)?;

        // Also ensure that a vote has not already been raised.
        ensure!(
            !NoConfidenceVotes::<T>::contains_key(project_key),
            Error::<T>::RoundStarted
        );

        // Create the accosiated vote struct, index can be used as an ensure on length has been called.
        let vote = Vote {
            yay: Default::default(),
            nay: contribution,
            // not using this so approved will be false.
            is_approved: false,
        };
        let now = frame_system::Pallet::<T>::block_number();
        // Create the accosiated round.
        let round = RoundOf::<T>::new(
            now,
            now + T::NoConfidenceTimeLimit::get(),
            vec![project_key],
            RoundType::VoteOfNoConfidence,
        );

        let round_key = RoundCount::<T>::get()
            .checked_add(1)
            .ok_or(Error::<T>::Overflow)?;
        // Insert the new round and votes into storage and update the RoundCount and UserVotes.
        NoConfidenceVotes::<T>::insert(project_key, vote);
        Rounds::<T>::insert(round_key, Some(round));
        RoundCount::<T>::mutate(|c| *c = c.saturating_add(1u32));
        UserVotes::<T>::insert((who, project_key, 0, round_key), true);
        Self::deposit_event(Event::NoConfidenceRoundCreated(round_key, project_key));

        Ok(())
    }

    /// Allows a contributer to agree or disagree with a vote of no confidence.
    /// Additional contributions after the vote is set are not counted and cannot be voted on again, todo?
    pub fn add_vote_no_confidence(
        who: T::AccountId,
        round_key: RoundKey,
        project_key: ProjectKey,
        is_yay: bool,
    ) -> DispatchResult {
        let round = Self::rounds(round_key).ok_or(Error::<T>::KeyNotFound)?;
        ensure!(
            round.project_keys.contains(&project_key),
            Error::<T>::ProjectNotInRound
        );
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let contribution = Self::ensure_contributor_of(&project, &who)?;

        let mut vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;
        ensure!(
            UserVotes::<T>::get((&who, project_key, 0, round_key)).is_none(),
            Error::<T>::VoteAlreadyExists
        );

        if is_yay {
            vote.yay = vote.yay.saturating_add(contribution);
        } else {
            vote.nay = vote.nay.saturating_add(contribution);
        }

        NoConfidenceVotes::<T>::insert(project_key, vote);
        UserVotes::<T>::insert((who, project_key, 0, round_key), true);

        Self::deposit_event(Event::NoConfidenceRoundVotedUpon(round_key, project_key));

        Ok(())
    }

    /// Called when a contributor wants to finalise a vote of no confidence.
    /// Votes for the vote of no confidence must reach the majority requred for the vote to pass.
    /// As defined in the config.
    /// This also calls a refund of funds to the users.
    pub fn call_finalise_no_confidence_vote(
        who: T::AccountId,
        round_key: RoundKey,
        project_key: ProjectKey,
        majority_required: u8,
    ) -> DispatchResultWithPostInfo {
        let mut round = Self::rounds(round_key).ok_or(Error::<T>::KeyNotFound)?;
        ensure!(
            round.project_keys.contains(&project_key),
            Error::<T>::ProjectNotInRound
        );
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        let _ = Self::ensure_contributor_of(&project, &who)?;
        let vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;

        let total_contribute = project.raised_funds;

        // 100 * Threshold =  (total_contribute * majority_required%)
        let threshold_votes: BalanceOf<T> =
            total_contribute.saturating_mul(majority_required.into());

        if vote.nay.saturating_mul(100u8.into()) >= threshold_votes {
            round.is_canceled = true;

            NoConfidenceVotes::<T>::remove(project_key);
            Rounds::<T>::insert(round_key, Some(round));
            // TODO: Need a sane bound on contributors in a project.
            // TODO: the same thing but with milestones.

            let locked_milestone_percentage = project.milestones.iter().fold(0, |acc, ms| {
                if !ms.1.is_approved {
                    acc.saturating_add(ms.1.percentage_to_unlock)
                } else {
                    acc
                }
            });

            let project_account_id = Self::project_account_id(project_key);

            match project.funding_type {
                FundingType::Brief | FundingType::Proposal => {
                    // Handle refunds on native chain, there is no need to deal with xcm here.
                    // Todo: Batch call using pallet-utility?
                    for (acc_id, contribution) in project.contributions.iter() {
                        let refund_amount: BalanceOf<T> = contribution
                            .value
                            .saturating_mul(locked_milestone_percentage.into())
                            / MAX_PERCENTAGE.into();
                        <T as Config>::MultiCurrency::transfer(
                            project.currency_id,
                            &project_account_id,
                            acc_id,
                            refund_amount,
                        )?;
                    }
                }
                FundingType::Grant(_) => {
                    let mut refund_amount: BalanceOf<T> = Default::default();
                    // Sum the contributions and send a single xcm.
                    for (_acc_id, contribution) in project.contributions.iter() {
                        let per_contributor = contribution
                            .value
                            .saturating_mul(locked_milestone_percentage.into())
                            / MAX_PERCENTAGE.into();
                        refund_amount = refund_amount.saturating_add(per_contributor);
                    }
                    <T as Config>::RefundHandler::send_refund_message_to_treasury(
                        project_account_id,
                        refund_amount,
                        project.currency_id,
                        project.funding_type,
                    )?;
                }
            }
            // Remove the project and return the storage deposit
            Self::reinstate_storage_deposit(&project.initiator)?;
            Projects::<T>::remove(project_key);

            Self::deposit_event(Event::NoConfidenceRoundFinalised(round_key, project_key));
        } else {
            return Err(Error::<T>::VoteThresholdNotMet.into());
        }
        Ok(().into())
    }

    // Called to ensure that an account is is a contributor to a project.
    fn ensure_contributor_of<'a>(
        project: &'a Project<T::AccountId, BalanceOf<T>, T::BlockNumber>,
        account_id: &'a T::AccountId,
    ) -> Result<BalanceOf<T>, Error<T>> {
        let contribution = project.contributions.get(account_id);
        match contribution {
            Some(c) => Ok(c.value),
            _ => Err(Error::<T>::OnlyContributorsCanVote),
        }
    }

    fn ensure_identity_is_decent(who: &T::AccountId) -> Result<(), Error<T>> {
        let identity =
            pallet_identity::Pallet::<T>::identity(who).ok_or(Error::<T>::IdentityNeeded)?;

        if identity
            .judgements
            .iter()
            .any(|j| j.1 == Judgement::Reasonable || j.1 == Judgement::KnownGood)
        {
            Ok(())
        } else {
            Err(Error::<T>::InvalidAccount)
        }
    }

    /// Call this to remove a project from storage and reinstate the deposit.
    fn reinstate_storage_deposit(who: &AccountIdOf<T>) -> DispatchResult {
        let _ = <T as Config>::MultiCurrency::unreserve(
            CurrencyId::Native,
            who,
            <T as Config>::ProjectStorageDeposit::get(),
        );
        Ok(())
    }
}
