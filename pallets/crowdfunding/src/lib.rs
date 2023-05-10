#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

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
	use pallet_proposals::{Milestone, ProposedMilestone, Vote, BoundedProposedMilestones, Contribution};
	use sp_core::H256;
	use common_types::{CurrencyId, FundingType};
	use orml_traits::MultiReservableCurrency;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type BoundedKeysPerRound<T> = BoundedVec<CrowdFundKeys, <T as Config>::MaxKeysPerRound>;
	type CrowdFundKey = u32;

    pub(crate) enum RoundType {
        ContributionRound
    }

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        type RoundExpiry: Get<BlockNumberFor<T>>;
        type MaxKeysPerRound: Get<u32>;
	}

    #[pallet::storage]
    pub type CrowdFundCount<T> = StorageValue<_, CrowdFundKey, ValueQuery>;

    #[pallet::storage]
	pub type CrowdFunds<T> = StorageMap<_, Blake_128, CrowdFundKey, CrowdFund, OptionQuery>;

    // TODO close rounds in hook.
    #[pallet::storage]
	pub type RoundsExpiring<T> = StorageMap<_, Blake_128, BlockNumberFor<T>, BoundedKeysPerRound<T>, ValueQuery>;

    /// Tracks wether CrowdFunds are in a given round type.
    /// Key 1 : CrowdFundID
    /// Key 2 : RoundType
    /// Value : Expiry BlockNumber
    #[pallet::storage]
	pub type CrowdFundsInRound<T> = StorageDoubleMap<_, Blake_128, CrowdFundKey, RoundType, BlockNumberFor<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CrowdFundCreated(
            T::AccountId,
            H256,
            CrowdFundKey,
            BalanceOf<T>,
            common_types::CurrencyId,
        ),
        // CrowdFund has been updated
        CrowdFundUpdated(T::AccountId, CrowdFundKey, BalanceOf<T>),
        FundingRoundCreated{crowdfund: CrowdFundKey}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Milestones must add up to 100.
		MilestonesTotalPercentageMustEqual100,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_crowdfund())]
        pub fn create_crowdfund(
            origin: OriginFor<T>,
            agreement_hash: H256,
            proposed_milestones: BoundedProposedMilestones<T>,
            required_funds: BalanceOf<T>,
            currency_id: common_types::CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Validation
            let total_percentage = proposed_milestones.iter().fold(0, |acc: u32, ms: &ProposedMilestone| acc.saturating_add(ms.percentage_to_unlock));
            ensure!(
                total_percentage == 100,
                Error::<T>::MilestonesTotalPercentageMustEqual100
            );

            let _ = Self::new_crowdfund(
                who,
                agreement_hash,
                proposed_milestones,
                required_funds,
                currency_id,
            )?;
            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_crowdfund())]
        pub fn update_crowdfund(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
            proposed_milestones: BoundedProposedMilestones<T>,
            required_funds: BalanceOf<T>,
            currency_id: CurrencyId,
            agreement_hash: H256,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let total_percentage = proposed_milestones.iter()
            .fold(0, |acc: u32, ms: &ProposedMilestone| acc.saturating_add(ms.percentage_to_unlock));

            ensure!(
                total_percentage == 100,
                Error::<T>::MilestonesTotalPercentageMustEqual100
            );

            Self::try_update_existing_crowdfund(
                // TODO: Optimise
                who.clone(),
                crowdfund_key,
                proposed_milestones,
                required_funds,
                currency_id,
                agreement_hash,
            )?;

            Self::deposit_event(Event::CrowdFundUpdated(who, crowdfund_key, required_funds));

            Ok(().into())
        }


        /// Step 1.5 (INITIATOR)
        /// Add whitelist to a crowdfund
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::add_crowdfund_whitelist())]
        pub fn add_crowdfund_whitelist(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
            new_whitelist_spots: BoundedWhitelistSpots<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_initiator(who, crowdfund_key)?;
            let mut crowdfund_whitelist_spots =
                WhitelistSpots::<T>::get(crowdfund_key).unwrap_or(BTreeMap::new());
            crowdfund_whitelist_spots.extend(new_whitelist_spots);
            <WhitelistSpots<T>>::insert(crowdfund_key, crowdfund_whitelist_spots);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::WhitelistAdded(crowdfund_key, now));
            Ok(().into())
        }

        /// Step 1.5 (INITIATOR)
        /// Remove a whitelist
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_crowdfund_whitelist())]
        pub fn remove_crowdfund_whitelist(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_initiator(who, crowdfund_key)?;
            <WhitelistSpots<T>>::remove(crowdfund_key);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::WhitelistRemoved(crowdfund_key, now));
            Ok(().into())
        }

		 /// Step 2 (ADMIN)
        /// Schedule a round
        /// crowdfund_keys: the crowdfunds were selected for this round
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::schedule_round())]
        pub fn open_contributions(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            ensure!(!crowdfund_key.is_empty(), Error::<T>::LengthMustExceedZero);
            ensure!(!CrowdFundsInRound::<T>::contains_key(crowdfund_key, RoundType::ContributionRound), Error::<T>::AlreadyInContributionRound);
            //todo: ensure it hasnt already had a contribution round?
            Self::start_contribution_round(start, end, crowdfund_keys, round_type)
            Self::deposit_event(Event::FundingRoundCreated(crowdfund_key))
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
	}

impl<T: Config> Pallet<T> {
	pub fn new_crowdfund(
        who: T::AccountId,
        agreement_hash: H256,
        proposed_milestones: BoundedProposedMilestones<T>,
        required_funds: BalanceOf<T>,
        currency_id: common_types::CurrencyId,
    ) -> Result<CrowdFundKey, DispatchError> {
        // Check if identity is required
        if <T as Config>::IsIdentityRequired::get() {
            let _ = Self::ensure_identity_is_decent(&who)?;
        }

        let crowdfund_key = CrowdFundCount::<T>::get();

        let mut milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();

        for (milestone, i) in proposed_milestones.iter().enumerate() {
            let milestone = Milestone {
                crowdfund_key,
                milestone_key,
                percentage_to_unlock: milestone.percentage_to_unlock,
                is_approved: false,
            };
            milestones.insert(i, milestone);
        }

        let crowdfund = Crowdfund {
            agreement_hash,
            milestones,
            contributions: BTreeMap::new(),
            required_funds,
            currency_id,
            raised_funds: (0_u32).into(),
            initiator: who.clone(),
            created_on: <frame_system::Pallet<T>>::block_number(),
            approved_for_funding: false,
            funding_threshold_met: false,
            cancelled: false,
        };

        // Add crowdfund to list
        <CrowdFunds<T>>::insert(crowdfund_key, crowdfund);

		let next_crowdfund_key = crowdfund_key.checked_add(1).ok_or(Error::<T>::Overflow)?;
        CrowdFundCount::<T>::put(next_crowdfund_key);

        Self::deposit_event(Event::CrowdFundCreated(
            who,
            agreement_hash,
            crowdfund_key,
            required_funds,
            currency_id,
        ));

        Ok(crowdfund_key)
    }

    pub fn try_update_existing_crowdfund(
        who: T::AccountId,
        crowdfund_key: CrowdFundKey,
        proposed_milestones: BoundedProposedMilestones<T>,
        required_funds: BalanceOf<T>,
        currency_id: CurrencyId,
        agreement_hash: H256,
    ) -> DispatchResultWithPostInfo {
        // Check if identity is required
        if <T as Config>::IsIdentityRequired::get() {
            let _ = Self::ensure_identity_is_decent(&who)?;
        }

        //check to ensure valid and existing crowdfund
        let mut crowdfund =
            CrowdFunds::<T>::get(&crowdfund_key).ok_or(Error::<T>::CrowdFundDoesNotExist)?;

        ensure!(crowdfund.initiator == who, Error::<T>::UserIsNotInitiator);

        ensure!(
            crowdfund.approved_for_funding == false,
            Error::<T>::CrowdFundAlreadyApproved
        );

        let mut milestone_key: u32 = 0;
        let mut milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();

        for milestone in proposed_milestones {
            let milestone = Milestone {
                crowdfund_key,
                milestone_key,
                percentage_to_unlock: milestone.percentage_to_unlock,
                is_approved: false,
            };
            milestones.insert(milestone_key.clone(), milestone.clone());
            milestone_key = milestone_key.checked_add(1).ok_or(Error::<T>::Overflow)?;
        }

        // Update crowdfund
        crowdfund.milestones = milestones;
        crowdfund.required_funds = required_funds;
        crowdfund.currency_id = currency_id;
        crowdfund.agreement_hash = agreement_hash;

        // Add crowdfund to list
        <CrowdFunds<T>>::insert(crowdfund_key, crowdfund);

        Ok(().into())
    }

    pub fn start_contribution_round(
        crowdfund_key: CrowdFundKey,
    ) -> DispatchResultWithPostInfo {

        let expiry_block = <T as frame_system::Config>::Pallet::<T>::block_number().saturating_add(<T as Config>::RoundExpiry::get());
        CrowdFundsInRound::try_mutate(expiry_block |list| -> DispatchResult {
            let _ = *list.try_insert(crowdfund_key)?;
            Ok(())    
        })?;
        CrowdFundsInRound::insert(crowdfund_key, RoundType::ContributionRound, expiry_block);
        CrowdFunds::<T>::try_mutate(crowdfund_key, |crowdfund| -> DispatchResult {
            if let Some(p) = crowdfund {
                p.approved_for_funding = true
            }
            Ok(())
        })?;
        Ok(().into())
    }

    pub fn new_contribution(
        who: T::AccountId,
        round_key: RoundKey,
        crowdfund_key: CrowdFundKey,
        value: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
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
            round.crowdfund_keys.contains(&crowdfund_key),
            Error::<T>::CrowdFundNotInRound
        );

        let mut crowdfund =
            CrowdFunds::<T>::get(&crowdfund_key).ok_or(Error::<T>::CrowdFundDoesNotExist)?;

        let new_amount = match crowdfund.contributions.get(&who) {
            Some(contribution) => contribution.value,
            None => BalanceOf::<T>::default(),
        }
        .saturating_add(value);

        // Find whitelist if exists
        if WhitelistSpots::<T>::contains_key(crowdfund_key) {
            let whitelist_spots = Self::whitelist_spots(crowdfund_key).ok_or(Error::<T>::WhiteListNotFound)?;
            ensure!(
                whitelist_spots.contains_key(&who.clone()),
                Error::<T>::OnlyWhitelistedAccountsCanContribute
            );

            let default_max_cap: BalanceOf<T> = (0u32).into();
            let max_cap = *whitelist_spots
                .get(&who.clone())
                .unwrap_or(&default_max_cap);

            ensure!(
                max_cap == default_max_cap || max_cap >= new_amount,
                Error::<T>::ContributionMustBeLowerThanMaxCap
            );
        }

        // Transfer contribute to crowdfund account
        T::MultiCurrency::transfer(
            crowdfund.currency_id,
            &who,
            &Self::crowdfund_account_id(crowdfund_key),
            value,
        )?;

        Self::deposit_event(Event::ContributeSucceeded(
            who.clone(),
            crowdfund_key,
            value,
            crowdfund.currency_id,
            now,
        ));

        let timestamp = <pallet_timestamp::Pallet<T>>::get();

        crowdfund.contributions.insert(
            who.clone(),
            Contribution {
                value: new_amount,
                timestamp,
            },
        );
        crowdfund.raised_funds = crowdfund.raised_funds.saturating_add(value);

        // Update storage item to include the new contributions.
        <CrowdFunds<T>>::insert(crowdfund_key, crowdfund.clone());

        Ok(().into())
    }

    pub fn do_approve(
        crowdfund_key: CrowdFundKey,
        round_key: RoundKey,
        milestone_keys: Option<BoundedMilestoneKeys<T>>,
    ) -> DispatchResultWithPostInfo {
        let round = Self::rounds(round_key).ok_or(Error::<T>::KeyNotFound)?;
        ensure!(
            round.crowdfund_keys.contains(&crowdfund_key),
            Error::<T>::CrowdFundNotInRound
        );
        ensure!(!round.is_canceled, Error::<T>::RoundCanceled);
        let now = <frame_system::Pallet<T>>::block_number();
        let mut crowdfund =
            CrowdFunds::<T>::get(&crowdfund_key).ok_or(Error::<T>::CrowdFundDoesNotExist)?;
        let total_contribution_amount: BalanceOf<T> = crowdfund.raised_funds;

        let funds_matched = total_contribution_amount >= crowdfund.required_funds;
        if !funds_matched {
            // If the funds have not been matched then check if the round is over
            ensure!(round.end < now, Error::<T>::RoundNotEnded);
            // TODO: PR for this exists.
            // Once the round ends, check for the funding threshold met. (set threshold for 75%)
        }
        crowdfund.funding_threshold_met = true;
        // Warning: This will allow the withdrawal of funds, approve is a governance action so should not be a problem.
        // Consider removing this/
		// TODO: move this into milestone pallet
        // if let Some(ms_keys) = milestone_keys {
        //     for milestone_key in ms_keys.into_iter() {
        //         ensure!(
        //             crowdfund.milestones.contains_key(&milestone_key),
        //             Error::<T>::MilestoneDoesNotExist
        //         );

        //         let vote_lookup_key = (crowdfund_key, milestone_key);

        //         let _ = MilestoneVotes::<T>::try_mutate(vote_lookup_key, |maybe_vote| {	
        //             if let Some(vote) = maybe_vote {
        //                 vote.is_approved = true;
        //             } else {
        //                 *maybe_vote = Some(Vote::default())
        //             }

        //             Ok::<(), Error<T>>(())
        //         })?;

        //         Self::deposit_event(Event::MilestoneApproved(
        //             crowdfund.initiator.clone(),
        //             crowdfund_key,
        //             milestone_key,
        //             now,
        //         ));
                
        //         let mut milestone = crowdfund.milestones.get_mut(&milestone_key).ok_or(Error::<T>::MilestoneDoesNotExist)?;
        //         milestone.is_approved = true;
        //     }
        // }
        <Rounds<T>>::insert(round_key, Some(round));
        <CrowdFunds<T>>::insert(crowdfund_key, crowdfund);
        Self::deposit_event(Event::CrowdFundApproved(round_key, crowdfund_key));
        Ok(().into())
    }
}
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Crowdfund<T: Config> {
		pub required_funds: Balance,
		pub approved_for_funding: bool,
		pub contributions: BTreeMap<AccountIdOf<T>, Contribution>,
		pub milestones: BTreeMap<u32, Milestone>,
		pub currency_id: CurrencyId,
		pub raised_funds: BalanceOf<T>,
		pub cancelled: bool,
		pub agreement_hash: H256,
		pub initiator: AccountIdOf<T>,
		pub created_on: BlockNumberFor<T>,
		pub funding_threshold_met: bool,
	}
}
