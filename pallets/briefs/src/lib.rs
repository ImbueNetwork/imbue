#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
mod integration_tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod test_utils;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use common_types::{milestone_origin::FundingType, CurrencyId};
    use frame_support::{pallet_prelude::*, sp_runtime::Saturating, traits::Get, BoundedBTreeMap};
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use pallet_proposals::traits::IntoProposal;
    use pallet_proposals::{Contribution, ProposedMilestone};
    use sp_core::{Hasher, H256};
    use sp_runtime::traits::Zero;
    use sp_std::convert::{From, TryInto};

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

    pub(crate) type BoundedBriefContributions<T> = BoundedBTreeMap<
        AccountIdOf<T>,
        Contribution<BalanceOf<T>, BlockNumberFor<T>>,
        <T as Config>::MaxBriefOwners,
    >;
    pub(crate) type BoundedProposedMilestones<T> =
        BoundedVec<ProposedMilestone, <T as Config>::MaxMilestonesPerBrief>;

    pub(crate) type BoundedBriefOwners<T> =
        BoundedVec<AccountIdOf<T>, <T as Config>::MaxBriefOwners>;

    pub type BriefHash = H256;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        /// The hasher used to generate the brief id.
        type BriefHasher: Hasher;

        type AuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The type that allows for evolution from brief to proposal.
        type IntoProposal: IntoProposal<AccountIdOf<Self>, BalanceOf<Self>, BlockNumberFor<Self>>;

        /// The maximum amount of owners to a brief.
        /// Also used to define the maximum contributions.
        type MaxBriefOwners: Get<u32>;

        type MaxMilestonesPerBrief: Get<u32>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn briefs)]
    pub type Briefs<T> =
        CountedStorageMap<_, Blake2_128Concat, BriefHash, BriefData<T>, OptionQuery>;

    /// The list of accounts approved to apply for work.
    /// Key: AccountId
    /// Value: Unit
    #[pallet::storage]
    #[pallet::getter(fn approved_accounts)]
    pub type FreelanceFellowship<T> =
        StorageMap<_, Blake2_128Concat, AccountIdOf<T>, (), ValueQuery>;

    /// The contributions to a brief, in a single currency.
    /// It's in a BTree to reduce storage call when we have to inevitably iterate the keys.
    /// Key 1: BriefHash
    /// Key 2: AccountIdOf<T>
    /// Value: Balance
    #[pallet::storage]
    #[pallet::getter(fn brief_contributions)]
    pub type BriefContributions<T> =
        StorageMap<_, Blake2_128Concat, BriefHash, BoundedBriefContributions<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        BriefSubmitted(T::AccountId, BriefHash),
        AccountApproved(AccountIdOf<T>),
        BriefEvolution(BriefHash),
        BriefContribution(T::AccountId, BriefHash),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The deposit you have sent is below the minimum requirement.
        DepositBelowMinimum,
        /// The bounty you have set is below the minimum requirement.
        BountyBelowMinimum,
        /// The contribution you have sent is more than the bounty total.
        ContributionMoreThanBounty,
        /// Only approved account can apply for briefs.
        OnlyApprovedAccountPermitted,
        /// Brief already exists in the block, please don't submit duplicates.
        BriefAlreadyExists,
        /// Brief not found.
        BriefNotFound,
        /// The BriefId generation failed.
        BriefHashingFailed,
        /// The bounty required for this brief has not been met.
        BountyTotalNotMet,
        /// There are too many briefs open for this block, try again later.
        BriefLimitReached,
        /// Currency must be set to add to a bounty.
        BriefCurrencyNotSet,
        /// Too many brief owners.
        TooManyBriefOwners,
        /// Not authorized to do this.
        NotAuthorised,
        /// The brief conversion failed.
        BriefConversionFailedGeneric,
        /// The brief has not yet been approved to commence by the freelancer.
        FreelancerApprovalRequired,
        /// Milestones total do not add up to 100%.
        MilestonesTotalPercentageMustEqual100,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Approve an account so that they can be accepted as an applicant.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::add_to_fellowship())]
        pub fn add_to_fellowship(
            origin: OriginFor<T>,
            account_id: AccountIdOf<T>,
        ) -> DispatchResult {
            <T as Config>::AuthorityOrigin::ensure_origin(origin)?;

            // Or if they are not voted by governance, be voted in by another approved freelancer?
            // TODO:
            FreelanceFellowship::<T>::insert(&account_id, ());
            Self::deposit_event(Event::<T>::AccountApproved(account_id));

            Ok(())
        }

        /// Create a brief to be funded or amended.
        /// In the current state the applicant must be approved.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::create_brief())]
        pub fn create_brief(
            origin: OriginFor<T>,
            mut brief_owners: BoundedBriefOwners<T>,
            applicant: AccountIdOf<T>,
            budget: BalanceOf<T>,
            initial_contribution: BalanceOf<T>,
            brief_id: BriefHash,
            currency_id: CurrencyId,
            milestones: BoundedProposedMilestones<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Briefs::<T>::get(brief_id).is_none(),
                Error::<T>::BriefAlreadyExists
            );

            /// <HB SBP Review:
            ///
            /// Re: sp_arithmetic library
            /// For the portion of the code below just acummulating the total percentage of the milestones with u32 seems to be enough,
            /// but using the sp_arithmetic library is a safer practice.
            ///
            /// >
            // Validation
            let total_percentage = milestones
                .iter()
                .fold(0u32, |acc: u32, ms: &ProposedMilestone| {
                    acc.saturating_add(ms.percentage_to_unlock)
                });

            ensure!(
                total_percentage == 100u32,
                Error::<T>::MilestonesTotalPercentageMustEqual100
            );

            if !brief_owners.contains(&who) {
                brief_owners
                    .try_push(who.clone())
                    .map_err(|_| Error::<T>::TooManyBriefOwners)?;
            }

            // TODO: freelancer fellowship handler
            // ensure!(
            //     FreelanceFellowship::<T>::contains_key(&applicant),
            //     Error::<T>::OnlyApprovedAccountPermitted
            // );

            /// <HB SBP Review:
            ///
            /// Usually balances reverves are fixed and determined at the runtime level since it is supposed to be a storage sanity measure.
            /// With the current design i could just reserve 0.000001 USD and that would be still chip to attack the network.
            /// As you are working in a multi-currency environment, i would suggest creating a new pallet that might define reserve values per currency.
            /// This new pallet would require root origin and it might be called from goverance chain.
            /// Or another option would be to only accept deposits in the native currency of the chain.
            ///
            /// >
            <T as Config>::RMultiCurrency::reserve(currency_id, &who, initial_contribution)?;

            if initial_contribution > Zero::zero() {
                BriefContributions::<T>::try_mutate(brief_id, |contributions| {
                    // This should never fail as the the bound is ensured when a brief is created.
                    let _ = contributions
                        .try_insert(
                            who.clone(),
                            Contribution {
                                value: initial_contribution,
                                timestamp: frame_system::Pallet::<T>::block_number(),
                            },
                        )
                        .map_err(|_| Error::<T>::TooManyBriefOwners)?;

                    Ok::<(), DispatchError>(())
                })?;
            }

            let brief = BriefData::new(
                brief_owners,
                budget,
                currency_id,
                frame_system::Pallet::<T>::block_number(),
                applicant,
                milestones,
            );

            Briefs::<T>::insert(brief_id, brief);

            Self::deposit_event(Event::<T>::BriefSubmitted(who, brief_id));

            Ok(())
        }

        /// Add a bounty to a brief.
        /// A bounty must be fully contributed to before a piece of work is started.
        ///
        /// TODO: runtime api to return how much bounty exactly is left on a brief.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::contribute_to_brief())]
        pub fn contribute_to_brief(
            origin: OriginFor<T>,
            brief_id: BriefHash,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let brief_record = Briefs::<T>::get(brief_id).ok_or(Error::<T>::BriefNotFound)?;
            // TODO: Minimum contribution.

            ensure!(
                brief_record.brief_owners.contains(&who),
                Error::<T>::NotAuthorised
            );

            /// <HB SBP Review:
            ///
            /// Same as the previous comment, please about reserves amount.
            /// >
            <T as Config>::RMultiCurrency::reserve(brief_record.currency_id, &who, amount)?;

            BriefContributions::<T>::try_mutate(brief_id, |contributions| {
                if let Some(contribution) = contributions.get_mut(&who) {
                    contribution.value = contribution.value.saturating_add(amount);
                    contribution.timestamp = frame_system::Pallet::<T>::block_number();
                } else {
                    // This should never fail as the the bound is ensured when a brief is created.
                    contributions
                        .try_insert(who.clone(), {
                            Contribution {
                                value: amount,
                                timestamp: frame_system::Pallet::<T>::block_number(),
                            }
                        })
                        .map_err(|_| Error::<T>::TooManyBriefOwners)?;
                }

                Ok::<(), DispatchError>(())
            })?;

            Self::deposit_event(Event::<T>::BriefContribution(who, brief_id));
            Ok(())
        }

        /// Once the freelancer is happy with both the milestones and the offering this can be called.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::commence_work())]
        pub fn commence_work(origin: OriginFor<T>, brief_id: BriefHash) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let brief = Briefs::<T>::get(brief_id).ok_or(Error::<T>::BriefNotFound)?;

            ensure!(who == brief.applicant, Error::<T>::NotAuthorised);

            let contributions = BriefContributions::<T>::get(brief_id);

            <T as Config>::IntoProposal::convert_to_proposal(
                brief.currency_id,
                contributions.into_inner(),
                brief_id,
                brief.applicant,
                brief.milestones.into(),
                FundingType::Brief,
            )
            .map_err(|_| Error::<T>::BriefConversionFailedGeneric)?;

            BriefContributions::<T>::remove(brief_id);
            Briefs::<T>::remove(brief_id);

            Self::deposit_event(Event::<T>::BriefEvolution(brief_id));
            Ok(())
        }
    }

    /// The data assocaited with a Brief
    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct BriefData<T: Config> {
        brief_owners: BoundedBriefOwners<T>,
        budget: BalanceOf<T>,
        currency_id: CurrencyId,
        created_at: BlockNumberFor<T>,
        applicant: AccountIdOf<T>,
        milestones: BoundedProposedMilestones<T>,
    }

    impl<T: Config> Pallet<T> {
        /// Used in the runtime api to quickly get the remainig funds as stated in the budget.
        pub fn get_remaining_bounty(brief_id: BriefHash) -> BalanceOf<T> {
            if let Some(brief) = Briefs::<T>::get(brief_id) {
                let sum: BalanceOf<T> = BriefContributions::<T>::get(brief_id)
                    .values()
                    .fold(Default::default(), |acc, x| acc.saturating_add(x.value));

                brief.budget.saturating_sub(sum)
            } else {
                Default::default()
            }
        }
    }

    impl<T: Config> BriefData<T> {
        pub fn new(
            brief_owners: BoundedBriefOwners<T>,
            budget: BalanceOf<T>,
            currency_id: CurrencyId,
            created_at: BlockNumberFor<T>,
            applicant: AccountIdOf<T>,
            milestones: BoundedProposedMilestones<T>,
        ) -> Self {
            Self {
                created_at,
                brief_owners,
                budget,
                currency_id,
                applicant,
                milestones,
            }
        }
    }
}
