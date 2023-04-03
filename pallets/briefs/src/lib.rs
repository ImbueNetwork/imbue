#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
mod integration_tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

    use common_types::CurrencyId;
    use frame_support::{pallet_prelude::*, sp_runtime::Saturating, traits::Get, BoundedBTreeMap};
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use pallet_proposals::traits::BriefEvolver;
    use pallet_proposals::{Contribution, ProposedMilestone, BoundedProposedMilestones};
    use sp_core::{Hasher, H256};
    use sp_std::convert::{From, TryInto};

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

    pub(crate) type BoundedBriefContributions<T> = BoundedBTreeMap<
        AccountIdOf<T>,
        Contribution<BalanceOf<T>, <T as pallet_timestamp::Config>::Moment>,
        <T as Config>::MaxBriefOwners,
    >;

    pub(crate) type BoundedBriefOwners<T> =
        BoundedVec<AccountIdOf<T>, <T as Config>::MaxBriefOwners>;

    pub type BriefHash = H256;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        /// The hasher used to generate the brief id.
        type BriefHasher: Hasher;

        type AuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The type that allows for evolution from brief to proposal.
        type BriefEvolver: BriefEvolver<
            AccountIdOf<Self>,
            BalanceOf<Self>,
            BlockNumberFor<Self>,
            <Self as pallet_timestamp::Config>::Moment,
        >;

        /// The maximum amount of owners to a brief.
        /// Also used to define the maximum contributions.
        type MaxBriefOwners: Get<u32>;
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
    /// Its in a BTree to reduce storage call when we have to inevitably iterate the keys.
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
        /// the bounty required for this brief has not been met.
        BountyTotalNotMet,
        /// There are too many briefs open for this block, try again later.
        BriefLimitReached,
        /// Currency must be set to add to a bounty.
        BriefCurrencyNotSet,
        /// Too many brief owners.
        TooManyBriefOwners,
        /// Not authorized to do this,
        NotAuthorised,
        /// The brief conversion failed
        BriefConversionFailedGeneric,
        /// The brief has not yet been approved to commence by the freelancer.
        FreelancerApprovalRequired,
        /// Milestones totals do not add up to 100%.
        MilestonesTotalPercentageMustEqual100,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Approve an account so that they can be accepted as an applicant.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn add_to_fellowship(
            origin: OriginFor<T>,
            account_id: AccountIdOf<T>,
        ) -> DispatchResult {
            <T as Config>::AuthorityOrigin::ensure_origin(origin)?;

            // Or if they are not voted by governance, be voted in by another approved freelancer?
            // todo.

            FreelanceFellowship::<T>::insert(&account_id, ());
            Self::deposit_event(Event::<T>::AccountApproved(account_id));

            Ok(())
        }

        /// Create a brief to be funded or amended.
        /// In the current state the applicant must be approved.
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn create_brief(
            origin: OriginFor<T>,
            mut brief_owners: BoundedBriefOwners<T>,
            applicant: AccountIdOf<T>,
            budget: BalanceOf<T>,
            initial_contribution: BalanceOf<T>,
            brief_id: BriefHash,
            currency_id: CurrencyId,
            milestones: BoundedProposedMilestones,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Briefs::<T>::get(brief_id).is_none(),
                Error::<T>::BriefAlreadyExists
            );

            // Validation
            let mut total_percentage = 0;
            for milestone in milestones.iter() {
                total_percentage += milestone.percentage_to_unlock;
            }
            ensure!(
                total_percentage == 100,
                Error::<T>::MilestonesTotalPercentageMustEqual100
            );

            if !brief_owners.contains(&who) {
                brief_owners
                    .try_push(who.clone())
                    .map_err(|_| Error::<T>::TooManyBriefOwners)?;
            }

            // todo freelancer fellowship handler
            // ensure!(
            //     FreelanceFellowship::<T>::contains_key(&applicant),
            //     Error::<T>::OnlyApprovedAccountPermitted
            // );

            <T as Config>::RMultiCurrency::reserve(currency_id, &who, initial_contribution)?;

            if initial_contribution > 0u32.into() {
                let _ = BriefContributions::<T>::try_mutate(&brief_id, |contributions| {
                    // this should never fail as the the bound is ensure when a brief is created.
                    let _ = contributions
                        .try_insert(
                            who.clone(),
                            Contribution {
                                value: initial_contribution,
                                timestamp: pallet_timestamp::Pallet::<T>::get(),
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
        /// Todo: runtime api to return how much bounty exactly is left on a brief.
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn contribute_to_brief(
            origin: OriginFor<T>,
            brief_id: BriefHash,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let brief_record = Briefs::<T>::get(&brief_id).ok_or(Error::<T>::BriefNotFound)?;
            // todo Minimum contribution.

            ensure!(
                brief_record.brief_owners.contains(&who),
                Error::<T>::NotAuthorised
            );

            <T as Config>::RMultiCurrency::reserve(brief_record.currency_id, &who, amount)?;

            let _ = BriefContributions::<T>::try_mutate(&brief_id, |contributions| {
                if let Some(val) = contributions.get_mut(&who) {
                    val.value = val.value.saturating_add(amount);
                    val.timestamp = pallet_timestamp::Pallet::<T>::get();
                } else {
                    // this should never fail as the the bound is ensure when a brief is created.
                    contributions
                        .try_insert(who.clone(), {
                            Contribution {
                                value: amount,
                                timestamp: pallet_timestamp::Pallet::<T>::get(),
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
        #[pallet::weight(10_000)]
        pub fn commence_work(origin: OriginFor<T>, brief_id: BriefHash) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let brief = Briefs::<T>::get(brief_id).ok_or(Error::<T>::BriefNotFound)?;

            ensure!(&who == &brief.applicant, Error::<T>::NotAuthorised);

            let contributions = BriefContributions::<T>::get(brief_id);

            <T as Config>::BriefEvolver::convert_to_proposal(
                brief.currency_id,
                contributions.into_inner(),
                brief_id.clone(),
                brief.applicant,
                brief.milestones.into(),
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
        milestones: BoundedProposedMilestones,
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
            milestones: BoundedProposedMilestones,
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
