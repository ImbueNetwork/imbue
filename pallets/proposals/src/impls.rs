use crate::*;
use common_types::milestone_origin::FundingType;
use scale_info::prelude::format;
use sp_runtime::traits::{Saturating, Zero};

impl<T: Config> Pallet<T> {
    /// The account ID of the fund pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    pub fn project_account_id(key: ProjectKey) -> AccountIdOf<T> {
        T::PalletId::get().into_sub_account_truncating(format!("//{key}"))
    }

    // Take a project and submit an associated milestone.
    pub(crate) fn new_milestone_submission(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
    ) -> DispatchResultWithPostInfo {
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        ensure!(project.initiator == who, Error::<T>::UserIsNotInitiator);
        let milestone = project
            .milestones
            .get(&milestone_key)
            .ok_or(Error::<T>::MilestoneDoesNotExist)?;
        ensure!(!milestone.is_approved, Error::<T>::MilestoneAlreadyApproved);

        let expiry_block =
            <T as Config>::MilestoneVotingWindow::get() + frame_system::Pallet::<T>::block_number();
        Rounds::<T>::insert(project_key, RoundType::VotingRound, expiry_block);
        RoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
            keys.try_push((project_key, RoundType::VotingRound, milestone_key))
                .map_err(|_| Error::<T>::Overflow)?;
            Ok::<(), DispatchError>(())
        })?;
        UserHasVoted::<T>::remove((project_key, RoundType::VotingRound, milestone_key));

        let vote = Vote::default();
        <MilestoneVotes<T>>::insert(project_key, milestone_key, vote);
        Self::deposit_event(Event::MilestoneSubmitted(who, project_key, milestone_key));
        Self::deposit_event(Event::VotingRoundCreated(project_key));
        Ok(().into())
    }

    pub(crate) fn new_milestone_vote(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
        approve_milestone: bool,
    ) -> DispatchResultWithPostInfo {
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        ensure!(
            Rounds::<T>::contains_key(project_key, RoundType::VotingRound),
            Error::<T>::VotingRoundNotStarted
        );
        let contribution_amount = project
            .contributions
            .get(&who)
            .ok_or(Error::<T>::OnlyContributorsCanVote)?
            .value;
        let now = frame_system::Pallet::<T>::block_number();
        let user_has_voted_key = (project_key, RoundType::VotingRound, milestone_key);

        UserHasVoted::<T>::try_mutate(user_has_voted_key, |votes| {
            ensure!(!votes.contains_key(&who), Error::<T>::VotesAreImmutable);
            votes
                .try_insert(who.clone(), approve_milestone)
                .map_err(|_| Error::<T>::Overflow)?;
            Ok::<(), DispatchError>(())
        })?;

        let vote: Vote<BalanceOf<T>> =
            MilestoneVotes::<T>::try_mutate(project_key, milestone_key, |vote| {
                if let Some(v) = vote {
                    if approve_milestone {
                        v.yay = v.yay.saturating_add(contribution_amount);
                    } else {
                        v.nay = v.nay.saturating_add(contribution_amount);
                    }
                    Ok::<Vote<BalanceOf<T>>, DispatchError>(v.clone())
                } else {
                    Err(Error::<T>::VotingRoundNotStarted.into())
                }
            })?;

        let funding_threshold: BalanceOf<T> =
            T::PercentRequiredForVoteToPass::get().mul_floor(project.raised_funds);

        Self::try_auto_finalise_milestone_voting(
            project_key,
            &vote,
            funding_threshold,
            user_has_voted_key,
            who.clone(),
        )?;
        Self::deposit_event(Event::VoteSubmitted(
            who,
            project_key,
            milestone_key,
            approve_milestone,
            now,
        ));
        Ok(().into())
    }

    pub(crate) fn new_withdrawal(
        who: T::AccountId,
        project_key: ProjectKey,
    ) -> DispatchResultWithPostInfo {
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        ensure!(!project.cancelled, Error::<T>::ProjectWithdrawn);
        ensure!(who == project.initiator, Error::<T>::UserIsNotInitiator);

        let mut unlocked_funds: BalanceOf<T> = Zero::zero();
        for (_, ms) in project.milestones.iter() {
            if ms.is_approved {
                let per_milestone = ms.percentage_to_unlock.mul_floor(project.raised_funds);
                unlocked_funds = unlocked_funds.saturating_add(per_milestone);
            }
        }

        let withdrawable: BalanceOf<T> = unlocked_funds.saturating_sub(project.withdrawn_funds);
        ensure!(
            withdrawable != Zero::zero(),
            Error::<T>::NoAvailableFundsToWithdraw
        );

        let fee = <T as Config>::ImbueFee::get().mul_floor(withdrawable);
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
                    <T as Config>::DepositHandler::return_deposit(p.deposit_id)?;
                    CompletedProjects::<T>::try_mutate(
                        &p.initiator,
                        |completed_projects| -> DispatchResult {
                            completed_projects
                                .try_push(project_key)
                                .map_err(|_| Error::<T>::TooManyProjects)?;
                            Ok(())
                        },
                    )?;
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

    /// This function raises a vote of no confidence.
    /// This round can only be called once and there after can only be voted on.
    pub(crate) fn raise_no_confidence_round(
        who: T::AccountId,
        project_key: ProjectKey,
    ) -> DispatchResult {
        //ensure that who is a contributor or root
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let contribution = project
            .contributions
            .get(&who)
            .ok_or(Error::<T>::OnlyContributorsCanVote)?;

        // Also ensure that a vote has not already been raised.
        ensure!(
            !NoConfidenceVotes::<T>::contains_key(project_key),
            Error::<T>::RoundStarted
        );

        let vote = Vote {
            yay: Zero::zero(),
            nay: contribution.value,
            is_approved: false,
        };

        let expiry_block = frame_system::Pallet::<T>::block_number()
            .saturating_add(<T as Config>::NoConfidenceTimeLimit::get());

        Rounds::<T>::insert(project_key, RoundType::VoteOfNoConfidence, expiry_block);
        RoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
            // The milestone key does not matter here as we are voting on the entire project.
            keys.try_push((project_key, RoundType::VoteOfNoConfidence, 0))
                .map_err(|_| Error::<T>::Overflow)?;
            Ok::<(), DispatchError>(())
        })?;
        UserHasVoted::<T>::try_mutate((project_key, RoundType::VoteOfNoConfidence, 0), |votes| {
            ensure!(!votes.contains_key(&who), Error::<T>::VotesAreImmutable);
            votes
                .try_insert(who.clone(), false)
                .map_err(|_| Error::<T>::Overflow)?;
            Ok::<(), DispatchError>(())
        })?;

        NoConfidenceVotes::<T>::insert(project_key, vote);
        Self::deposit_event(Event::NoConfidenceRoundCreated(who, project_key));
        Ok(())
    }

    /// Allows a contributer to agree or disagree with a vote of no confidence.
    pub(crate) fn add_vote_no_confidence(
        who: T::AccountId,
        project_key: ProjectKey,
        is_yay: bool,
    ) -> DispatchResult {
        ensure!(
            Rounds::<T>::contains_key(project_key, RoundType::VoteOfNoConfidence),
            Error::<T>::ProjectNotInRound
        );
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let contribution = project
            .contributions
            .get(&who)
            .ok_or(Error::<T>::OnlyContributorsCanVote)?;

        let nay_vote = NoConfidenceVotes::<T>::try_mutate(project_key, |maybe_vote| {
            if let Some(v) = maybe_vote {
                if is_yay {
                    v.yay = v.yay.saturating_add(contribution.value);
                } else {
                    v.nay = v.nay.saturating_add(contribution.value);
                }
                Ok::<BalanceOf<T>, DispatchError>(v.nay)
            } else {
                Err(Error::<T>::VotingRoundNotStarted.into())
            }
        })?;

        UserHasVoted::<T>::try_mutate((project_key, RoundType::VoteOfNoConfidence, 0), |votes| {
            ensure!(!votes.contains_key(&who), Error::<T>::VotesAreImmutable);
            votes
                .try_insert(who.clone(), false)
                .map_err(|_| Error::<T>::Overflow)?;
            Ok::<(), DispatchError>(())
        })?;

        Self::deposit_event(Event::NoConfidenceRoundVotedUpon(who.clone(), project_key));

        //once the voting is complete check if the confidence vote could be auto finalized
        //getting the total threshold required for the total confidence
        let voting_no_confidence_threshold: BalanceOf<T> =
            T::PercentRequiredForVoteNoConfidenceToPass::get().mul_floor(project.raised_funds);

        //verifying whether the no confidence vote has passed the threshold if so then auto finalize it
        if nay_vote >= voting_no_confidence_threshold {
            let locked_milestone_percentage =
                project.milestones.iter().fold(Percent::zero(), |acc, ms| {
                    if !ms.1.is_approved {
                        acc.saturating_add(ms.1.percentage_to_unlock)
                    } else {
                        acc
                    }
                });

            let project_account_id = Self::project_account_id(project_key);

            match project.funding_type {
                FundingType::Proposal => {
                    // Handle refunds on native chain, there is no need to deal with xcm here.
                    for (acc_id, contribution) in project.contributions.iter() {
                        let refund_amount =
                            locked_milestone_percentage.mul_floor(contribution.value);
                        <T as Config>::MultiCurrency::transfer(
                            project.currency_id,
                            &project_account_id,
                            acc_id,
                            refund_amount,
                        )?;
                    }
                }

                FundingType::Brief => {
                    //Have to handle it in the dispute pallet
                }

                // Must a grant be treasury funded?
                FundingType::Grant(_) => {
                    let mut refund_amount: BalanceOf<T> = Zero::zero();
                    // Sum the contributions and send a single xcm.
                    for (_acc_id, contribution) in project.contributions.iter() {
                        let per_contributor =
                            locked_milestone_percentage.mul_floor(contribution.value);
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
            Projects::<T>::remove(project_key);
            Rounds::<T>::remove(project_key, RoundType::VoteOfNoConfidence);
            <T as Config>::DepositHandler::return_deposit(project.deposit_id)?;
            Self::deposit_event(Event::NoConfidenceRoundFinalised(who, project_key));
        }
        Ok(())
    }

    #[deprecated(since = "3.1.0", note = "autofinalisation has been implemented.")]
    pub(crate) fn _call_finalise_no_confidence_vote(
        who: T::AccountId,
        project_key: ProjectKey,
        majority_required: Percent,
    ) -> DispatchResultWithPostInfo {
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        ensure!(
            Rounds::<T>::contains_key(project_key, RoundType::VoteOfNoConfidence),
            Error::<T>::ProjectNotInRound
        );
        ensure!(
            project.contributions.contains_key(&who),
            Error::<T>::OnlyContributorsCanVote
        );

        let vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;
        let threshold_votes: BalanceOf<T> = majority_required.mul_floor(project.raised_funds);

        if vote.nay >= threshold_votes {
            let locked_milestone_percentage =
                project.milestones.iter().fold(Percent::zero(), |acc, ms| {
                    if !ms.1.is_approved {
                        acc.saturating_add(ms.1.percentage_to_unlock)
                    } else {
                        acc
                    }
                });

            let project_account_id = Self::project_account_id(project_key);

            // TODO: this should be generic and not bound to funding type..
            match project.funding_type {
                FundingType::Brief | FundingType::Proposal => {
                    //
                    // Handle refunds on native chain, there is no need to deal with xcm here.
                    // Todo: Batch call using pallet-utility?
                    for (acc_id, contribution) in project.contributions.iter() {
                        let refund_amount =
                            locked_milestone_percentage.mul_floor(contribution.value);
                        <T as Config>::MultiCurrency::transfer(
                            project.currency_id,
                            &project_account_id,
                            acc_id,
                            refund_amount,
                        )?;
                    }
                }
                // Must a grant be treasury funded?
                FundingType::Grant(_) => {
                    let mut refund_amount: BalanceOf<T> = Zero::zero();
                    // Sum the contributions and send a single xcm.
                    for (_acc_id, contribution) in project.contributions.iter() {
                        let per_contributor =
                            locked_milestone_percentage.mul_floor(contribution.value);
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

            Projects::<T>::remove(project_key);
            <T as Config>::DepositHandler::return_deposit(project.deposit_id)?;
            Self::deposit_event(Event::NoConfidenceRoundFinalised(who, project_key));
        } else {
            return Err(Error::<T>::VoteThresholdNotMet.into());
        }
        Ok(().into())
    }

    pub(crate) fn try_auto_finalise_milestone_voting(
        project_key: ProjectKey,
        vote: &Vote<BalanceOf<T>>,
        funding_threshold: BalanceOf<T>,
        user_has_voted_key: (ProjectKey, RoundType, MilestoneKey),
        who: AccountIdOf<T>,
    ) -> Result<(), DispatchError> {
        // If the yay votes is over the funding threshold then the milestone is approved.
        if vote.yay >= funding_threshold {
            Projects::<T>::mutate(project_key, |maybe_project| {
                if let Some(p) = maybe_project {
                    if let Some(ms) = p.milestones.get_mut(&user_has_voted_key.2) {
                        ms.is_approved = true
                    }
                }
            });

            Self::deposit_event(Event::MilestoneApproved(
                who,
                project_key,
                user_has_voted_key.2,
                <frame_system::Pallet<T>>::block_number(),
            ));

            Self::close_voting_round(project_key, user_has_voted_key)?;
        }

        if vote.nay >= funding_threshold {
            Self::close_voting_round(project_key, user_has_voted_key)?;
            Self::deposit_event(Event::MilestoneRejected(
                user_has_voted_key.0,
                user_has_voted_key.2,
            ));
        }
        Ok(())
    }

    pub(crate) fn close_voting_round(
        project_key: ProjectKey,
        user_has_voted_key: (ProjectKey, RoundType, MilestoneKey),
    ) -> Result<(), DispatchError> {
        // Prevent further voting.
        let exp_block = Rounds::<T>::take(project_key, RoundType::VotingRound)
            .ok_or(Error::<T>::VotingRoundNotStarted)?;
        // Prevent hook from calling.
        RoundsExpiring::<T>::remove(exp_block);
        // Allow future votes to occur on this milestone
        UserHasVoted::<T>::remove(user_has_voted_key);
        Ok(())
    }
}