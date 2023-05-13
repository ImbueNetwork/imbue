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
	use frame_support::{
        pallet_prelude::*,
        BoundedBTreeMap,
        transactional
    };
	use frame_system::pallet_prelude::*;
	use pallet_proposals::{Milestone, ProposedMilestone, Vote};
	use sp_core::H256;
	use common_types::{CurrencyId, FundingType};
	use orml_traits::{MultiReservableCurrency, MultiCurrency};
    use pallet_proposals::traits::IntoProposal;
    use sp_std::collections::btree_map::BTreeMap;
    use pallet_identity::Judgement;
    use frame_support::sp_runtime::Saturating;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    
    pub type BoundedKeysPerRound<T> = BoundedVec<CrowdFundKey, <T as Config>::MaxKeysPerRound>;
    pub type BoundedContributions<T> = BoundedBTreeMap<AccountIdOf<T>, Contribution<T>, <T as Config>::MaxContributionsPerCrowdFund>;
    pub type BoundedMilestoneKeys<T> = BoundedVec<MilestoneKey, <T as Config>::MaxMilestonesPerCrowdFund>;
    pub type BoundedMilestones<T> = BoundedBTreeMap<MilestoneKey, Milestone, <T as Config>::MaxMilestonesPerCrowdFund>;
    pub type BoundedWhitelistSpots<T> = BoundedBTreeMap<AccountIdOf<T>, BalanceOf<T>, <T as Config>::MaxWhitelistPerCrowdFund>;
    pub type BoundedProposedMilestones<T> = BoundedVec<ProposedMilestone, <T as Config>::MaxMilestonesPerCrowdFund>;

    pub type CrowdFundKey = u32;
    pub type MilestoneKey = u32;

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    pub enum RoundType {
        ContributionRound
    }

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_identity::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        type RoundExpiry: Get<BlockNumberFor<Self>>;
        type MaxKeysPerRound: Get<u32>;
        type MaxContributionsPerCrowdFund: Get<u32>;
        type MaxMilestonesPerCrowdFund: Get<u32>;
        type MaxWhitelistPerCrowdFund: Get<u32>;
        type IsIdentityRequired: Get<bool>;
        type AuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        type MinimumRequiredFunds: Get<BalanceOf<Self>>;
        //type IntoProposals: IntoProposal;
	}

    /// The count of crowdfunds, used as an id.
    #[pallet::storage]
    pub type CrowdFundCount<T> = StorageValue<_, CrowdFundKey, ValueQuery>;

    /// Stores a list of crowdfunds.
    #[pallet::storage]
	pub type CrowdFunds<T> = StorageMap<_, Blake2_128, CrowdFundKey, CrowdFund<T>, OptionQuery>;

    /// Stores the crowdfund keys that are expiring on a given block.
    /// Handled in the hooks,
    #[pallet::storage]
	pub type RoundsExpiring<T> = StorageMap<_, Blake2_128, BlockNumberFor<T>, BoundedKeysPerRound<T>, ValueQuery>;

    /// Tracks wether CrowdFunds are in a given round type.
    /// Key 1 : CrowdFundID
    /// Key 2 : RoundType
    /// Value : Expiry BlockNumber
    #[pallet::storage]
	pub type CrowdFundsInRound<T> = StorageDoubleMap<_, Blake2_128, CrowdFundKey, Blake2_128, RoundType, BlockNumberFor<T>, ValueQuery>;

    /// Tracks the whitelists of a given crowdfund.
    #[pallet::storage]
    #[pallet::getter(fn whitelist_spots)]
    pub type WhitelistSpots<T: Config> =
        StorageMap<_, Identity, CrowdFundKey, BoundedWhitelistSpots<T>, OptionQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A crowdfund has been created.
        CrowdFundCreated(
            T::AccountId,
            H256,
            CrowdFundKey,
            BalanceOf<T>,
            common_types::CurrencyId,
        ),
        /// CrowdFund has been updated
        CrowdFundUpdated(T::AccountId, CrowdFundKey, BalanceOf<T>),
        /// A funding round was created
        FundingRoundCreated(CrowdFundKey),
        /// A whitelist was removed.
        WhitelistRemoved(CrowdFundKey),
        /// A whitelist was added.
        WhitelistAdded(CrowdFundKey, BlockNumberFor<T>),
        /// Contribution successful.
        ContributeSucceeded(T::AccountId, CrowdFundKey, BalanceOf<T>),
        /// A crowdfund has been approved.
        CrowdFundApproved(CrowdFundKey)

	}

	#[pallet::error]
	pub enum Error<T> {
		/// Milestones must add up to 100.
		MilestonesTotalPercentageMustEqual100,
        /// Your contribution is below the minimum.
        ContributionTooLow,
        /// The contribution round has not been started.
        ContributionRoundNotStarted,
        /// This crowdfund has reached the maximum contributions.
        TooManyContributions,
        /// The funds required to approve the crowdfund has not been reached.
        RequiredFundsNotReached,
        /// The crowdfund key you specified does not exist.
        CrowdFundDoesNotExist,
        /// This crowdfunding is already in a contribution round.
        AlreadyInContributionRound,
        /// There was an overflow prevented in pallet-crowdfunding.
        Overflow,
        /// You must be the initator to call this.
        UserIsNotInitiator,
        /// This crowdfund has already been approved.
        CrowdFundAlreadyApproved,
        /// Your account is not good enough for this.
        BadJudgement,
        /// The whitelist was not found.
        WhiteListNotFound,
        /// Only whitelist accounts can contribute.
        OnlyWhitelistedAccountsCanContribute,
        /// The total contribution must be lower than the max cap.
        ContributionMustBeLowerThanMaxCap,
        /// An identity is required for this.
        IdentityNeeded,
        /// Below the minimum required funds.
        BelowMinimumRequiredFunds,

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
        #[pallet::weight(10000)]
        pub fn create_crowdfund(
            origin: OriginFor<T>,
            agreement_hash: H256,
            proposed_milestones: BoundedProposedMilestones<T>,
            required_funds: BalanceOf<T>,
            currency_id: common_types::CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(required_funds >= <T as Config>::MinimumRequiredFunds::get(), Error::<T>::BelowMinimumRequiredFunds);
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

        // TODO: Use options
        #[pallet::call_index(1)]
        #[pallet::weight(10000)]
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
        #[pallet::weight(10000)]
        pub fn add_crowdfund_whitelist(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
            new_whitelist_spots: BoundedWhitelistSpots<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_initiator(who, crowdfund_key)?;
            let crowdfund_whitelist_spots =
                WhitelistSpots::<T>::get(crowdfund_key).unwrap_or(BTreeMap::new().try_into().expect("Empty BTree is always smaller than bound; qed"));
            
            let mut unbounded = crowdfund_whitelist_spots.into_inner();
            unbounded.extend(new_whitelist_spots); 

            let bounded: BoundedWhitelistSpots<T> = unbounded.try_into().map_err(|_|Error::<T>::Overflow)?;
            <WhitelistSpots<T>>::insert(crowdfund_key, bounded);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::WhitelistAdded(crowdfund_key, now));
            Ok(().into())
        }

        /// Step 1.5 (INITIATOR)
        /// Remove a whitelist
        #[pallet::call_index(3)]
        #[pallet::weight(10000)]
        pub fn remove_crowdfund_whitelist(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_initiator(who, crowdfund_key)?;
            <WhitelistSpots<T>>::remove(crowdfund_key);
            Self::deposit_event(Event::WhitelistRemoved(crowdfund_key));
            Ok(().into())
        }

		/// Step 2 (ADMIN)
        /// Open a round for contributions, this must be called before contributions are allowed.
        #[pallet::call_index(4)]
        #[pallet::weight(10000)]
        pub fn open_contributions(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
            // should governance define the contribution time?
            //length: BlockNumberFor<T>,
        ) -> DispatchResultWithPostInfo {
            <T as Config>::AuthorityOrigin::ensure_origin(origin)?;
            ensure!(!CrowdFundsInRound::<T>::contains_key(crowdfund_key, RoundType::ContributionRound), Error::<T>::AlreadyInContributionRound);
            //todo: ensure it hasnt already had a contribution round?
            let _ = Self::start_contribution_round(crowdfund_key)?;
            Self::deposit_event(Event::FundingRoundCreated(crowdfund_key));

            Ok(().into())
        }

        /// Step 3 (CONTRIBUTOR/FUNDER)
        /// Contribute to a crowdfund
        #[pallet::call_index(6)]
        #[pallet::weight(10000)]
        #[transactional]
        pub fn contribute(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let _ = Self::new_contribution(&who, crowdfund_key, value)?;
            Self::deposit_event(Event::ContributeSucceeded(
                who.clone(),
                crowdfund_key,
                value,
            ));
            Ok(().into())
        }

        /// Step 4 (ADMIN)
        /// Approve crowdfund
        /// If the crowdfund is approved, the crowdfund initiator can withdraw funds for approved milestones
        #[pallet::call_index(7)]
        #[pallet::weight(10000)]
        pub fn approve_crowdfund_for_milestone_submission(
            origin: OriginFor<T>,
            crowdfund_key: CrowdFundKey,
        ) -> DispatchResultWithPostInfo {
            T::AuthorityOrigin::ensure_origin(origin)?;
            let _ = Self::do_approve(crowdfund_key)?;
            Ok(().into())
        }

	}

#[pallet::hooks]
impl  <T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(n: BlockNumberFor<T>) -> Weight {
        let mut weight: Weight = Default::default(); 
        let crowdfund_keys: BoundedKeysPerRound<T> = RoundsExpiring::<T>::take(n);
        weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

        // SAFETY: BoundedKeysPerRound must be sane as to not have overweight blocks.
        crowdfund_keys.iter().for_each(|key| {
            CrowdFundsInRound::<T>::remove(key, RoundType::ContributionRound);
            weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        });
        
        weight
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

        // Todo: Take storage deposit>

        // For now we keep them as proposed milestones until the project is able to submit.
        let crowdfund = CrowdFund {
            agreement_hash,
            milestones: proposed_milestones,
            contributions: BTreeMap::new().try_into().expect("empty BTree is smaller than bound; qed"),
            required_funds,
            currency_id,
            raised_funds: (0_u32).into(),
            initiator: who.clone(),
            created_on: <frame_system::Pallet<T>>::block_number(),
            approved_for_funding: false,
            cancelled: false,
            is_converted: false,
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

        // Update crowdfund
        crowdfund.milestones = proposed_milestones;
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

        let expiry_block = frame_system::Pallet::<T>::block_number().saturating_add(<T as Config>::RoundExpiry::get());
        RoundsExpiring::<T>::try_mutate(expiry_block, |list| -> DispatchResult {
            let _ = list.try_push(crowdfund_key).map_err(|_|Error::<T>::Overflow)?;
            Ok(())    
        })?;
        CrowdFundsInRound::<T>::insert(crowdfund_key, RoundType::ContributionRound, expiry_block);
        CrowdFunds::<T>::try_mutate(crowdfund_key, |crowdfund| -> DispatchResult {
            if let Some(p) = crowdfund {
                p.approved_for_funding = true
            }
            Ok(())
        })?;
        Ok(().into())
    }

    pub fn new_contribution<'a>(
        who: &'a T::AccountId,
        crowdfund_key: CrowdFundKey,
        additional_amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        // TODO add configurable value for min and max contribution per contributor
        ensure!(additional_amount > (0_u32).into(), Error::<T>::ContributionTooLow);
        // ensure is in round and if exists expiry is less than now
        ensure!(CrowdFundsInRound::<T>::contains_key(crowdfund_key, RoundType::ContributionRound), Error::<T>::ContributionRoundNotStarted);
        let crowdfund =
            CrowdFunds::<T>::get(&crowdfund_key).ok_or(Error::<T>::CrowdFundDoesNotExist)?;

        let new_value = match crowdfund.contributions.get(&who) {
            Some(contribution) => contribution.value,
            None => Default::default()
        }.saturating_add(additional_amount);
        

        // Find whitelist if exists
        if WhitelistSpots::<T>::contains_key(crowdfund_key) {
            let whitelist_spots = Self::whitelist_spots(crowdfund_key).ok_or(Error::<T>::WhiteListNotFound)?;
            ensure!(
                whitelist_spots.contains_key(&who.clone()),
                Error::<T>::OnlyWhitelistedAccountsCanContribute
            );

            let max_cap = *whitelist_spots
                .get(&who.clone())
                .unwrap_or(&Default::default());

            ensure!(
                max_cap >= new_value,
                Error::<T>::ContributionMustBeLowerThanMaxCap
            );
        }
        
        // Reserve amount to be used later.
        T::MultiCurrency::reserve(
            crowdfund.currency_id,
            &who,
            additional_amount,
        )?;

        CrowdFunds::<T>::try_mutate(crowdfund_key, |crowdfund| {
            if let Some(cf) = crowdfund {
                
                let cont = Contribution {
                    created_on: frame_system::Pallet::<T>::block_number(),
                    value: new_value,
                };
                // Just write over the previous if exists.
                // There is probably a more sophisticated way of doing this.
                let _ = cf.contributions.try_insert(who.clone(), cont).map_err(|_| Error::<T>::TooManyContributions)?;
                cf.raised_funds = cf.raised_funds.saturating_add(additional_amount); 
            }
            Ok::<(), DispatchError>(())
        })?;

        Ok(().into())
    }

    pub fn do_approve(
        crowdfund_key: CrowdFundKey,
    ) -> DispatchResultWithPostInfo {

        let now = <frame_system::Pallet<T>>::block_number();
        let mut crowdfund =
            CrowdFunds::<T>::get(&crowdfund_key).ok_or(Error::<T>::CrowdFundDoesNotExist)?;
        let funds_matched = ensure!(crowdfund.raised_funds >= crowdfund.required_funds, Error::<T>::RequiredFundsNotReached);

        
        crowdfund.is_converted = true;

        // call IntoProposal

        Self::deposit_event(Event::CrowdFundApproved(crowdfund_key));
        Ok(().into())
    }

    pub fn ensure_initiator(who: T::AccountId, crowdfund_key: CrowdFundKey) -> Result<(), Error<T>> {
        let crowdfund = CrowdFunds::<T>::get(&crowdfund_key).ok_or(Error::<T>::CrowdFundDoesNotExist)?;
        match crowdfund.initiator == who {
            true => Ok(()),
            false => Err(Error::<T>::UserIsNotInitiator),
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
            Err(Error::<T>::BadJudgement)
        }
    }
}
	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct CrowdFund<T: Config> {
		pub required_funds: BalanceOf<T>,
		pub approved_for_funding: bool,
		pub contributions: BoundedContributions<T>,
		pub milestones: BoundedProposedMilestones<T>,
		pub currency_id: CurrencyId,
		pub raised_funds: BalanceOf<T>,
		pub cancelled: bool,
		pub agreement_hash: H256,
		pub initiator: AccountIdOf<T>,
		pub created_on: BlockNumberFor<T>,
		pub is_converted: bool,
	}

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
    pub struct Contribution<T: Config> {
        value: BalanceOf<T>,
        created_on: BlockNumberFor<T>,
    }

    // Called to ensure that an account is is a contributor to a crowdfund.

}



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
        