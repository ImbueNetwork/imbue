
use crate::*;

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

    pub fn new_project(
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

    pub fn new_round(
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

        // Add proposal round to list
        <Rounds<T>>::insert(key, Some(round));

        for project_key in project_keys.iter() {
            let project =
                Projects::<T>::get(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

            // Update project as approved for funding, assuming only RoundType::Contribution will be used.
            let updated_project = Project {
                approved_for_funding: true,
                ..project
            };

            // Update storage with the new project.
            <Projects<T>>::insert(project_key, updated_project);
        }

        match round_type {
            RoundType::VotingRound => {Self::deposit_event(Event::VotingRoundCreated(key, project_keys.to_vec()))},
            RoundType::ContributionRound => {Self::deposit_event(Event::FundingRoundCreated(key, project_keys.to_vec()))},
            _ => {}
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
            if !round.is_canceled 
            && round.start < now 
            && round.end > now
            && round.round_type == RoundType::ContributionRound
            {
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
        let _round = processing_round.ok_or(Error::<T>::RoundNotProcessing)?;
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

        // Update storage item to include the new contributions.
        <Projects<T>>::insert(project_key, project);

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
            milestones,
            ..project
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
            milestones,
            ..project
        };
        // Add project to list
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
            withdrawn_funds: available_funds + project.withdrawn_funds,
            ..project
        };
        // Add project to list
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
            cancelled: true,
            ..project
        };
        // Updated new project status to chain
        <Projects<T>>::insert(project_key, updated_project);
        Self::deposit_event(Event::ProjectLockedFundsRefunded(
            project_key,
            refunded_funds,
        ));

        Ok(().into())
    }

    /// This function raises a vote of no confidence.
    /// This round can only be called once and there after can only be voted on.
    /// The person calling it must be a contributor.
    pub fn call_raise_no_confidence_round(who: T::AccountId, project_key: ProjectKey) -> DispatchResult {

        //ensure that who is a contributor or root
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let contributor = Self::ensure_contributor_of(&project, &who)?;

        // Also ensure that a vote has not already been raised.
        ensure!(!NoConfidenceVotes::<T>::contains_key(project_key), Error::<T>::RoundStarted);

        // Create the accosiated vote struct, index can be used as an ensure on length has been called.
        let vote = Vote {
            yay: Default::default(),
            nay: contributor.value,
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

        let round_key = RoundCount::<T>::get();
        // Insert the new round and votes into storage and update the RoundCount and UserVotes.
        NoConfidenceVotes::<T>::insert(project_key, vote);
        Rounds::<T>::insert(round_key, Some(round));
        RoundCount::<T>::mutate(|c| {*c += 1u32});
        UserVotes::<T>::insert((who, project_key, 0, round_key), true);
        
        Self::deposit_event(Event::NoConfidenceRoundCreated(
            round_key,
            project_key,
        ));

        Ok(()).into()
    }
    
    /// Allows a contributer to agree or disagree with a vote of no confidence.
    /// Additional contributions after the vote is set are not counted and cannot be voted on again, todo?
    pub fn call_add_vote_no_confidence(who: T::AccountId, project_key: ProjectKey, is_yay: bool) -> DispatchResult {
        // Ensure that who is a contributor.
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
        let contributor = Self::ensure_contributor_of(&project, &who)?;

        // Ensure that the vote has been raised.
        let mut vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;

        // We need to find the round key and the only current way is finding the round.
        let mut round_key = 0u32;
        let mut round: Option<RoundOf<T>> = None;
        let round_count = RoundCount::<T>::get();

        for i in (0..round_count).rev() {
            // Get the current round and check that both the key exists and the value under the key is some.
            let current_round = Self::rounds(i).ok_or(Error::<T>::KeyNotFound)?;
            if !current_round.is_canceled 
            && current_round.project_keys.contains(&project_key) 
            && current_round.round_type == RoundType::VoteOfNoConfidence
            && current_round.end >= frame_system::Pallet::<T>::block_number()
            {
                round = Some(current_round);
                round_key = i;
                break;
            }
        }
        // Ensure a round has been found + that they have not already voted.
        ensure!(round.is_some(), Error::<T>::RoundNotProcessing);
        ensure!(UserVotes::<T>::get((&who, project_key, 0, round_key)).is_none(), Error::<T>::VoteAlreadyExists);

        // Update the vote
            if is_yay {
                vote.yay += contributor.value 
            } else {
                vote.nay += contributor.value
            }
        
        // Insert new vote.
        NoConfidenceVotes::<T>::insert(project_key, vote);

        // Insert person who has voted.
        UserVotes::<T>::insert((who, project_key, 0, round_key), true);

        Self::deposit_event(Event::NoConfidenceRoundVotedUpon(
            round_key,
            project_key,
        ));

        Ok(()).into()
    }

    /// Called when a contributor wants to finalise a vote of no confidence.
    /// Votes for the vote of no confidence must reach the majority requred for the vote to pass.
    pub fn call_finalise_no_confidence_vote(who: T::AccountId, project_key: ProjectKey, majority_required: u8) -> DispatchResultWithPostInfo {
        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;

        // Ensure that the caller is a contributor and that the vote has been raised.
        let _ = Self::ensure_contributor_of(&project, &who)?;
        let vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;
        
        // We need to find the round key and the only current way is finding the round.
        let mut round: Option<RoundOf<T>> = None;
        let round_count = RoundCount::<T>::get();
        // This does not have to be an option as round is.
        let mut round_key: RoundKey = 0;
        
        for i in (0..round_count).rev() {
            // Get the current round and check that both the key exists and the value under the key is some.
            let current_round = Self::rounds(i).ok_or(Error::<T>::KeyNotFound)?;

            if !current_round.is_canceled 
            && current_round.project_keys.contains(&project_key) 
            && current_round.round_type == RoundType::VoteOfNoConfidence
            && current_round.end >= frame_system::Pallet::<T>::block_number()
            {
                round = Some(current_round);
                round_key = i;
                break;
            }
        }
        ensure!(round.is_some(), Error::<T>::RoundNotProcessing);

        // The nay vote must >= minimum threshold required for the vote to pass.
        let total_contribute = Self::get_total_project_contributions(project_key)?;
        
        // 100 * Threshold =  (total_contribute * majority_required)/100
        let threshold_votes: BalanceOf<T> = total_contribute * majority_required.into();

        if vote.nay * 100u8.into() >= threshold_votes {
            // Vote of no confidence has passed alas refund. 
            round.as_mut().expect("is_some() has been called; qed").is_canceled = true;

            // Set Round to is cancelled, remove the vote from NoConfidenceVotes, and do the refund.
            NoConfidenceVotes::<T>::remove(project_key);
            Rounds::<T>::insert(round_key, round);
            let _ = Self::do_refund(project_key)?;

            Self::deposit_event(Event::NoConfidenceRoundFinalised(
                round_key,
                project_key,
            ));

        } else {
            return Err(Error::<T>::VoteThresholdNotMet.into())
        }
        Ok(().into())
    }

    // Called to ensure that an account is is a contributor to a project.
    fn ensure_contributor_of<'a>(
        project: &'a Project<T::AccountId, BalanceOf<T>,T::BlockNumber>,
        account_id: &'a T::AccountId
    ) -> Result<&'a Contribution<T::AccountId, BalanceOf<T>>, Error<T>> {
        let maybe_contributor: Vec<&Contribution<T::AccountId, BalanceOf<T>>> = 
        project.contributions
        .iter()
        .filter(|acc| &acc.account_id == account_id)
        .collect();
        ensure!(maybe_contributor.len() > 0, Error::<T>::InvalidAccount);

        Ok(maybe_contributor[0])
    }
}
