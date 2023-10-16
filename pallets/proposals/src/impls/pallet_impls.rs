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
        Rounds::<T>::insert(
            (project_key, milestone_key),
            RoundType::VotingRound,
            expiry_block,
        );
        RoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
            keys.try_push((project_key, RoundType::VotingRound, milestone_key))
                .map_err(|_| Error::<T>::Overflow)?;
            Ok::<(), DispatchError>(())
        })?;

        IndividualVoteStore::<T>::try_mutate(project_key, |maybe_votes| {
            if let Some(individual_votes) = maybe_votes {
                individual_votes.clear_milestone_votes(milestone_key);
            } else {
                return Err(Error::<T>::IndividualVoteNotFound.into());
            };
            Ok::<(), DispatchError>(())
        })?;

        MilestoneVotes::<T>::try_mutate(project_key, |vote_btree| {
            vote_btree
                .try_insert(milestone_key, Vote::default())
                .map_err(|_| Error::<T>::TooManyMilestoneVotes)?;

            Ok::<(), DispatchError>(())
        })?;

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
            Rounds::<T>::contains_key((project_key, milestone_key), RoundType::VotingRound),
            Error::<T>::VotingRoundNotStarted
        );
        let contribution_amount = project
            .contributions
            .get(&who)
            .ok_or(Error::<T>::OnlyContributorsCanVote)?
            .value;
        let now = frame_system::Pallet::<T>::block_number();
        let user_has_voted_key = (project_key, RoundType::VotingRound, milestone_key);

        IndividualVoteStore::<T>::try_mutate(project_key, |maybe_individual_votes| {
            if let Some(individual_votes) = maybe_individual_votes {
                individual_votes.insert_individual_vote(milestone_key, &who, approve_milestone)?;
            }
            Ok::<(), DispatchError>(())
        })?;

        let vote: Vote<BalanceOf<T>> =
            MilestoneVotes::<T>::try_mutate(project_key, |vote_btree| {
                if let Some(vote) = vote_btree.get_mut(&milestone_key) {
                    if approve_milestone {
                        vote.yay = vote.yay.saturating_add(contribution_amount);
                    } else {
                        vote.nay = vote.nay.saturating_add(contribution_amount);
                    }
                    Ok::<Vote<BalanceOf<T>>, DispatchError>(vote.clone())
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

            Self::close_voting_round(project_key, user_has_voted_key)?;

            Self::deposit_event(Event::MilestoneApproved(
                who,
                project_key,
                user_has_voted_key.2,
                <frame_system::Pallet<T>>::block_number(),
            ));
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
        let exp_block =
            Rounds::<T>::take((project_key, user_has_voted_key.2), RoundType::VotingRound)
                .ok_or(Error::<T>::VotingRoundNotStarted)?;
        // Prevent hook from calling.
        RoundsExpiring::<T>::remove(exp_block);
        // Allow future votes to occur on this milestone
        IndividualVoteStore::<T>::try_mutate(project_key, |maybe_individual_votes| {
            if let Some(individual_votes) = maybe_individual_votes {
                individual_votes.clear_milestone_votes(user_has_voted_key.2);
            } else {
                return Err(Error::<T>::IndividualVoteNotFound.into());
            }
            Ok::<(), DispatchError>(())
        })?;

        Ok(())
    }
}
