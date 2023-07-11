#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod test_utils;

#[cfg(test)]
mod migrations;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use common_types::{milestone_origin::FundingType, CurrencyId, TreasuryOrigin};
    use frame_support::{pallet_prelude::*, BoundedVec};
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use pallet_proposals::{traits::IntoProposal, Contribution, ProposedMilestone};
    use sp_arithmetic::{per_things::Percent, traits::One};
    use sp_core::H256;
    use sp_runtime::Saturating;
    use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

    pub(crate) type BoundedPMilestones<T> =
        BoundedVec<ProposedMilestone, <T as Config>::MaxMilestonesPerGrant>;
    pub(crate) type BoundedApprovers<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxApprovers>;
    pub type GrantId = H256;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Maximum amount of milestones per grant.
        type MaxMilestonesPerGrant: Get<u32>;
        /// The maximum approvers for a given grant.
        type MaxApprovers: Get<u32>;
        type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        /// The type that converts into a proposal for milestone submission.
        type IntoProposal: IntoProposal<AccountIdOf<Self>, BalanceOf<Self>, BlockNumberFor<Self>>;
        type WeightInfo: WeightInfo;
    }

    /// Stores all the grants a user has submitted.
    /// Key 1: AccountId
    /// Key 2: GrantId
    /// Value: ()
    #[pallet::storage]
    pub type GrantsSubmittedBy<T: Config> =
        StorageDoubleMap<_, Blake2_128, AccountIdOf<T>, Blake2_128, GrantId, (), ValueQuery>;

    /// Used to check wether a grant_id has already been submitted.
    #[pallet::storage]
    pub type GrantsSubmitted<T: Config> = StorageMap<_, Blake2_128Concat, GrantId, (), ValueQuery>;

    #[pallet::storage]
    pub type GrantCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        GrantSubmitted {
            submitter: AccountIdOf<T>,
            grant_id: GrantId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Milestones must sum to 100
        MustSumTo100,
        /// The GrantId specified cannot be found.
        GrantNotFound,
        /// The grant already exists.
        GrantAlreadyExists,
        /// There are too many milestones.
        TooManyMilestones,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Instead of iterating, create a project from the parameters of a grant.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::create_and_convert())]
        pub fn create_and_convert(
            origin: OriginFor<T>,
            proposed_milestones: BoundedPMilestones<T>,
            assigned_approvers: BoundedApprovers<T>,
            currency_id: CurrencyId,
            amount_requested: BalanceOf<T>,
            treasury_origin: TreasuryOrigin,
            grant_id: GrantId,
        ) -> DispatchResultWithPostInfo {
            let submitter = ensure_signed(origin)?;

            let percentage_sum = proposed_milestones
                .iter()
                .fold(Default::default(), |acc: Percent, x| {
                    acc.saturating_add(x.percentage_to_unlock)
                });
            ensure!(percentage_sum == One::one(), Error::<T>::MustSumTo100);
            ensure!(
                !GrantsSubmitted::<T>::contains_key(grant_id),
                Error::<T>::GrantAlreadyExists
            );

            let mut contributions = BTreeMap::new();
            let _ = assigned_approvers
                .iter()
                .map(|approver_id| {
                    contributions.insert(
                        approver_id.clone(),
                        Contribution {
                            value: amount_requested / (assigned_approvers.len() as u32).into(),
                            timestamp: frame_system::Pallet::<T>::block_number(),
                        },
                    )
                })
                .collect::<Vec<_>>();

            <T as Config>::IntoProposal::convert_to_proposal(
                currency_id,
                contributions,
                grant_id,
                submitter.clone(),
                proposed_milestones
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyMilestones)?,
                FundingType::Grant(treasury_origin),
            )?;

            GrantsSubmittedBy::<T>::insert(&submitter, &grant_id, ());
            GrantsSubmitted::<T>::insert(&grant_id, ());
            Self::deposit_event(Event::<T>::GrantSubmitted {
                grant_id,
                submitter,
            });
            Ok(().into())
        }
    }
}
