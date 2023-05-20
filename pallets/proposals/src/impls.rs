use crate::*;
use common_types::milestone_origin::FundingType;
use pallet_identity::Judgement;
use sp_runtime::traits::Saturating;
use sp_std::{collections::btree_map::BTreeMap, vec};
use scale_info::prelude::format;
use crate::Error::*;

pub const MAX_PERCENTAGE: u32 = 100u32;

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

    pub fn project_account_id(key: ProjectKey) -> AccountIdOf<T> {
        T::PalletId::get().into_sub_account_truncating(format!("//{key}"))
    }

    // Take a project and submit an associated milestone.
    pub fn new_milestone_submission(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
    ) -> DispatchResultWithPostInfo {
        let now = <frame_system::Pallet<T>>::block_number();
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        // Ensure that only the initiator has submitted
        ensure!(project.initiator == who, Error::<T>::UserIsNotInitiator);
        ensure!(!MilestoneVotes::<T>::contains_key((project_key, milestone_key)), Error::<T>::VoteAlreadyExists);
        ensure!(project.milestones.contains_key(&milestone_key), Error::<T>::MilestoneDoesNotExist);

        let expiry_block = <T as Config>::MilestoneVotingWindow::get() + frame_system::Pallet::<T>::block_number();
        Rounds::<T>::insert(project_key, RoundType::VotingRound, expiry_block);
        RoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
            keys.try_push((project_key, RoundType::VotingRound)).map_err(|_| Error::<T>::Overflow)?;
            Ok::<(), DispatchError>(())
        })?;
        let vote = Vote::default();
        let vote_lookup_key = (project_key, milestone_key);
        <MilestoneVotes<T>>::insert(vote_lookup_key, vote);

        Self::deposit_event(Event::MilestoneSubmitted(who, project_key, milestone_key));
        Self::deposit_event(Event::VotingRoundCreated(project_key));
        Ok(().into())
    }

    pub fn new_milestone_vote(
        who: T::AccountId,
        project_key: ProjectKey,
        milestone_key: MilestoneKey,
        approve_milestone: bool,
    ) -> DispatchResultWithPostInfo {

        let mut project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let round = Rounds::<T>::get(project_key, RoundType::VotingRound).ok_or(Error::<T>::KeyNotFound)?;
        let contribution_amount = project.contributions.get(&who).ok_or(Error::<T>::OnlyContributorsCanVote)?.value;
        let vote_lookup_key = (who.clone(), project_key, milestone_key, RoundType::VotingRound);
        let now = frame_system::Pallet::<T>::block_number();

        ensure!(!UserVotes::<T>::contains_key(&vote_lookup_key), Error::<T>::VoteAlreadyExists);
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
            //TODO: put this into a function, its too messy.

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
            Error::<T>::InvalidAccount
        );
        // TODO: this is also messy with the mut reference, clean up
        let mut milestone = project.milestones.get_mut(&milestone_key).ok_or(Error::<T>::MilestoneDoesNotExist)?;

        // set is_approved
        let vote_lookup_key = (project_key, milestone_key);
        let vote = Self::milestone_votes(vote_lookup_key).ok_or(Error::<T>::KeyNotFound)?;

        // let the 100 x threshold required = total_votes * majority required
        let threshold_votes: BalanceOf<T> = project
            .raised_funds
            .saturating_mul(T::PercentRequiredForVoteToPass::get().into());
        let percent_multiple: BalanceOf<T> = 100u32.into();

        ensure!(
            percent_multiple.saturating_mul(vote.yay.saturating_add(vote.nay)) >= threshold_votes,
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
                    // TODO: reinstate storage deposit
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
    /// The person calling it must be a contributor.
    pub fn raise_no_confidence_round(who: T::AccountId, project_key: ProjectKey) -> DispatchResult {
        //ensure that who is a contributor or root
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let contribution = project.contributions.get(&who).ok_or(Error::<T>::KeyNotFound)?;

        // Also ensure that a vote has not already been raised.
        ensure!(
            !NoConfidenceVotes::<T>::contains_key(project_key),
            Error::<T>::RoundStarted
        );

        // Create the accosiated vote struct, index can be used as an ensure on length has been called.
        let vote = Vote {
            yay: Default::default(),
            nay: contribution.value,
            // not using this so approved will be false.
            is_approved: false,
        };

        let expiry_block = frame_system::Pallet::<T>::block_number().saturating_add(<T as Config>::NoConfidenceTimeLimit::get());

        Rounds::<T>::insert(project_key, RoundType::VoteOfNoConfidence, expiry_block);
        RoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
            keys.try_push((project_key, RoundType::VoteOfNoConfidence)).map_err(|_| Error::<T>::Overflow)?;
            Ok::<(), DispatchError>(())
        })?;
        NoConfidenceVotes::<T>::insert(project_key, vote);
        Self::deposit_event(Event::NoConfidenceRoundCreated(project_key));
        Ok(())
    }

    /// Allows a contributer to agree or disagree with a vote of no confidence.
    /// Additional contributions after the vote is set are not counted and cannot be voted on again, todo?
    pub fn add_vote_no_confidence(
        who: T::AccountId,
        project_key: ProjectKey,
        is_yay: bool,
    ) -> DispatchResult {
        ensure!(Rounds::<T>::contains_key(project_key, RoundType::VoteOfNoConfidence), ProjectNotInRound::<T>);

        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        let contribution = project.contributions.get(&who).ok_or(Error::<T>::KeyNotFound)?;
        let mut vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;
        
        ensure!(
            !UserVotes::<T>::contains_key((&who, project_key, 0, RoundType::VoteOfNoConfidence)),
            Error::<T>::VoteAlreadyExists
        );

        if is_yay {
            vote.yay = vote.yay.saturating_add(contribution.value);
        } else {
            vote.nay = vote.nay.saturating_add(contribution.value);
        }

        NoConfidenceVotes::<T>::insert(project_key, vote);
        UserVotes::<T>::insert((who, project_key, 0, RoundType::VoteOfNoConfidence), true);

        Self::deposit_event(Event::NoConfidenceRoundVotedUpon(project_key));
        Ok(())
    }

    /// Called when a contributor wants to finalise a vote of no confidence.
    /// Votes for the vote of no confidence must reach the majority requred for the vote to pass.
    /// As defined in the config.
    /// This also calls a refund of funds to the users.
    pub fn call_finalise_no_confidence_vote(
        who: T::AccountId,
        project_key: ProjectKey,
        majority_required: u8,
    ) -> DispatchResultWithPostInfo {
        ensure!(Rounds::<T>::contains_key(project_key, RoundType::VoteOfNoConfidence), ProjectNotInRound::<T>);
        let project = Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        ensure!(project.contributions.contains_key(&who), Error::<T>::OnlyContributorsCanVote);

        let vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;

        let total_contribute = project.raised_funds;

        // 100 * Threshold =  (total_contribute * majority_required%)
        let threshold_votes: BalanceOf<T> =
            total_contribute.saturating_mul(majority_required.into());

        if vote.nay.saturating_mul(100u8.into()) >= threshold_votes {
            NoConfidenceVotes::<T>::remove(project_key);
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

            Projects::<T>::remove(project_key);
            Self::deposit_event(Event::NoConfidenceRoundFinalised(project_key));
        } else {
            return Err(Error::<T>::VoteThresholdNotMet.into());
        }
        Ok(().into())
    }
}
