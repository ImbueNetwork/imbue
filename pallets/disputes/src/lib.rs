#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use frame_support::traits::ReservableCurrency;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
    use sp_std::convert::TryInto;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	//This holds the votes when a no confidence round is raised.
	//#[pallet::storage]
	//#[pallet::getter(fn no_confidence_votes)]
	//pub(super) type NoConfidenceVotes<T: Config> =
	//	StorageMap<_, Identity, ProjectKey, Vote<BalanceOf<T>>, OptionQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored { something: u32, who: T::AccountId },
		// You have created a vote of no confidence.
        //NoConfidenceRoundCreated(RoundKey, ProjectKey),
        // You have voted upon a round of no confidence.
        //NoConfidenceRoundVotedUpon(RoundKey, ProjectKey),
        // You have finalised a vote of no confidence.
        //NoConfidenceRoundFinalised(RoundKey, ProjectKey),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
	}


	// Currently unimplemented
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		        // Vote on an already existing "Vote of no condidence" round.
        // is_yay is FOR the project's continuation.
        // so is_yay = false == against the project from continuing perhaps should be flipped.
        // #[pallet::call_index(13)]
        // #[pallet::weight(<T as Config>::WeightInfo::vote_on_no_confidence_round())]
        // pub fn vote_on_no_confidence_round(
        //     origin: OriginFor<T>,
        //     round_key: Option<RoundKey>,
        //     project_key: ProjectKey,
        //     is_yay: bool,
        // ) -> DispatchResult {
        //     let who = ensure_signed(origin)?;
        //     let voting_round_key = round_key.unwrap_or(RoundCount::<T>::get());
        //     Self::add_vote_no_confidence(who, voting_round_key, project_key, is_yay)
        // }

        // /// Finalise a "vote of no condidence" round.
        // /// Votes must pass a threshold as defined in the config trait for the vote to succeed.
        // #[pallet::call_index(14)]
        // #[pallet::weight(<T as Config>::WeightInfo::finalise_no_confidence_round())]
        // pub fn finalise_no_confidence_round(
        //     origin: OriginFor<T>,
        //     round_key: Option<RoundKey>,
        //     project_key: ProjectKey,
        // ) -> DispatchResultWithPostInfo {
        //     let who = ensure_signed(origin)?;
        //     let voting_round_key = round_key.unwrap_or(RoundCount::<T>::get());
        //     Self::call_finalise_no_confidence_vote(
        //         who,
        //         voting_round_key,
        //         project_key,
        //         T::PercentRequiredForVoteToPass::get(),
        //     )
        // }
		// In case of contributors losing confidence in the initiator a "Vote of no confidence" can be called.
        // This will start a round which each contributor can vote on.
        // The round will last as long as set in the Config.
        //#[pallet::call_index(12)]
        //#[pallet::weight(<T as Config>::WeightInfo::raise_vote_of_no_confidence())]
        //pub fn raise_vote_of_no_confidence(
        //    origin: OriginFor<T>,
        //    project_key: ProjectKey,
        //) -> DispatchResult {
        //    let who = ensure_signed(origin)?;
        //    Self::raise_no_confidence_round(who, project_key)
        //}
	//}
}
}
// This function raises a vote of no confidence.
    // This round can only be called once and there after can only be voted on.
//    // The person calling it must be a contributor.
//    pub fn raise_no_confidence_round(who: T::AccountId, project_key: ProjectKey) -> DispatchResult {
//        //ensure that who is a contributor or root
//        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
//        let contribution = Self::ensure_contributor_of(&project, &who)?;
//
//        // Also ensure that a vote has not already been raised.
//        ensure!(
//            !NoConfidenceVotes::<T>::contains_key(project_key),
//            Error::<T>::RoundStarted
//        );
//
//        // Create the accosiated vote struct, index can be used as an ensure on length has been called.
//        let vote = Vote {
//            yay: Default::default(),
//            nay: contribution,
//            // not using this so approved will be false.
//            is_approved: false,
//        };
//        let now = frame_system::Pallet::<T>::block_number();
//        // Create the accosiated round.
//        let round = RoundOf::<T>::new(
//            now,
//            now + T::NoConfidenceTimeLimit::get(),
//            vec![project_key],
//            RoundType::VoteOfNoConfidence,
//        );
//
//        let round_key = RoundCount::<T>::get()
//            .checked_add(1)
//            .ok_or(Error::<T>::Overflow)?;
//        // Insert the new round and votes into storage and update the RoundCount and UserVotes.
//        NoConfidenceVotes::<T>::insert(project_key, vote);
//        Rounds::<T>::insert(round_key, Some(round));
//        RoundCount::<T>::mutate(|c| *c += 1u32);
//        UserVotes::<T>::insert((who, project_key, 0, round_key), true);
//        Self::deposit_event(Event::NoConfidenceRoundCreated(round_key, project_key));
//
//        Ok(()).into()
//    }
//
//     Allows a contributer to agree or disagree with a vote of no confidence.
//     Additional contributions after the vote is set are not counted and cannot be voted on again, todo?
//    pub fn add_vote_no_confidence(
//        who: T::AccountId,
//        round_key: RoundKey,
//        project_key: ProjectKey,
//        is_yay: bool,
//    ) -> DispatchResult {
//        let round = Self::rounds(round_key).ok_or(Error::<T>::KeyNotFound)?;
//        ensure!(
//            round.project_keys.contains(&project_key),
//            Error::<T>::ProjectNotInRound
//        );
//        // Ensure that who is a contributor.
//        let project = Self::projects(project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
//        let contribution = Self::ensure_contributor_of(&project, &who)?;
//
//        // Ensure that the vote has been raised.
//        let mut vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;
//        // Ensure a round has been found + that they have not already voted.
//        ensure!(
//            UserVotes::<T>::get((&who, project_key, 0, round_key)).is_none(),
//            Error::<T>::VoteAlreadyExists
//        );
//
//        // Update the vote
//        if is_yay {
//            vote.yay += contribution
//        } else {
//            vote.nay += contribution
//        }
//
//        // Insert new vote.
//        NoConfidenceVotes::<T>::insert(project_key, vote);
//
//        // Insert person who has voted.
//        UserVotes::<T>::insert((who, project_key, 0, round_key), true);
//
//        Self::deposit_event(Event::NoConfidenceRoundVotedUpon(round_key, project_key));
//
//        Ok(()).into()
//    }
//
//    // Called when a contributor wants to finalise a vote of no confidence.
//    // Votes for the vote of no confidence must reach the majority requred for the vote to pass.
//    // As defined in the config.
//    // This also calls a refund of funds to the users.
//    pub fn call_finalise_no_confidence_vote(
//        who: T::AccountId,
//        round_key: RoundKey,
//        project_key: ProjectKey,
//        majority_required: u8,
//    ) -> DispatchResultWithPostInfo {
//        let mut round = Self::rounds(round_key).ok_or(Error::<T>::KeyNotFound)?;
//        ensure!(
//            round.project_keys.contains(&project_key),
//            Error::<T>::ProjectNotInRound
//        );
//        let project = Projects::<T>::get(&project_key).ok_or(Error::<T>::ProjectDoesNotExist)?;
//
//        // Ensure that the caller is a contributor and that the vote has been raised.
//        let _ = Self::ensure_contributor_of(&project, &who)?;
//        let vote = NoConfidenceVotes::<T>::get(project_key).ok_or(Error::<T>::NoActiveRound)?;
//
//        // The nay vote must >= minimum threshold required for the vote to pass.
//        let total_contribute = project.raised_funds;
//
//        // 100 * Threshold =  (total_contribute * majority_required)/100
//        let threshold_votes: BalanceOf<T> = total_contribute * majority_required.into();
//
//        if vote.nay * 100u8.into() >= threshold_votes {
//            // Vote of no confidence has passed alas refund.
//            round.is_canceled = true;
//            // Set Round to is cancelled, remove the vote from NoConfidenceVotes, and do the refund.
//            NoConfidenceVotes::<T>::remove(project_key);
//            Rounds::<T>::insert(round_key, Some(round));
//            let _ = Self::add_refunds_to_queue(project_key)?;
//
//            Self::deposit_event(Event::NoConfidenceRoundFinalised(round_key, project_key));
//        } else {
//            return Err(Error::<T>::VoteThresholdNotMet.into());
//        }
//        Ok(().into())
//    }
//