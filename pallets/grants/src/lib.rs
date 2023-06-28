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

pub mod weights;
pub use weights::*;

mod migrations;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use common_types::{milestone_origin::FundingType, CurrencyId, TreasuryOrigin};
    use frame_support::{dispatch::fmt::Debug, pallet_prelude::*, BoundedVec};
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use pallet_deposits::traits::DepositHandler;
    use pallet_proposals::{traits::IntoProposal, Contribution, ProposedMilestone};
    use sp_arithmetic::per_things::Percent;
    use sp_core::H256;
    use sp_runtime::Saturating;
    use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

    pub(crate) type BoundedPMilestones<T> =
        BoundedVec<ProposedMilestone, <T as Config>::MaxMilestonesPerGrant>;
    pub(crate) type BoundedApprovers<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxApprovers>;
    pub(crate) type GrantId = H256;
    pub(crate) type DepositIdOf<T> =
        <<T as Config>::DepositHandler as DepositHandler<BalanceOf<T>, AccountIdOf<T>>>::DepositId;
    pub(crate) type StorageItemOf<T> = <<T as Config>::DepositHandler as DepositHandler<
        BalanceOf<T>,
        AccountIdOf<T>,
    >>::StorageItem;

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
        /// The authority allowed to cancel a pending grant.
        type CancellingAuthority: EnsureOrigin<Self::RuntimeOrigin>;

        /// The storage item is used to generate the deposit_id.
        type GrantStorageItem: Get<StorageItemOf<Self>>;
        type DepositHandler: DepositHandler<BalanceOf<Self>, AccountIdOf<Self>>;

        type WeightInfo: WeightInfo;
    }

    /// Stores all the Grants waiting for approval, funding and eventual conversion into milestones.
    /// Key 1: GrantId
    /// Value: Grant<T>
    #[pallet::storage]
    pub type PendingGrants<T: Config> = StorageMap<_, Blake2_128Concat, GrantId, Grant<T>, OptionQuery>;

    /// Stores all the grants a user has submitted.
    /// Key 1: AccountId
    /// Key 2: GrantId
    /// Value: ()
    #[pallet::storage]
    pub type GrantsSubmittedBy<T: Config> =
        StorageDoubleMap<_, Blake2_128, AccountIdOf<T>, Blake2_128, GrantId, (), ValueQuery>;

    #[pallet::storage]
    pub type GrantCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        GrantSubmitted {
            submitter: AccountIdOf<T>,
            grant_id: GrantId,
        },
        GrantEdited {
            grant_id: GrantId,
        },
        GrantCancelled {
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
        /// Overflow Error in pallet-grants.
        Overflow,
        /// Only the submitter can edit this grant.
        OnlySubmitterCanEdit,
        /// Cannot use a cancelled grant.
        GrantCancelled,
        /// This grant has already been converted.
        AlreadyConverted,
        /// The conversion to proposals failed.
        GrantConversionFailedGeneric,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::default()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// A grant starts here with nothing agreed upon and
        /// probably awaiting much back and forth.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_initial_grant())]
        pub fn submit_initial_grant(
            origin: OriginFor<T>,
            //ipfs_hash: [u8; 32],
            proposed_milestones: BoundedPMilestones<T>,
            assigned_approvers: BoundedApprovers<T>,
            currency_id: CurrencyId,
            amount_requested: BalanceOf<T>,
            treasury_origin: TreasuryOrigin,
            grant_id: GrantId,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;

            let total_percentage = proposed_milestones
                .iter()
                .fold(Percent::zero(), |acc: Percent, x| {
                    acc.saturating_add(x.percentage_to_unlock)
                });
            ensure!(total_percentage.is_one(), Error::<T>::MustSumTo100);

            ensure!(
                !PendingGrants::<T>::contains_key(grant_id),
                Error::<T>::GrantAlreadyExists
            );

            let deposit_id = T::DepositHandler::take_deposit(
                submitter.clone(),
                T::GrantStorageItem::get(),
                CurrencyId::Native,
            )?;

            let grant = Grant {
                milestones: proposed_milestones,
                submitter: submitter.clone(),
                approvers: assigned_approvers,
                // ipfs_hash,
                created_on: frame_system::Pallet::<T>::block_number(),
                is_cancelled: false,
                is_converted: false,
                currency_id,
                amount_requested,
                treasury_origin,
                deposit_id,
            };

            PendingGrants::<T>::insert(grant_id, grant);
            GrantsSubmittedBy::<T>::insert(&submitter, grant_id, ());
            GrantCount::<T>::mutate(|count| {
                *count = count.saturating_add(1);
            });

            Self::deposit_event(Event::<T>::GrantSubmitted {
                submitter,
                grant_id,
            });
            Ok(())
        }

        /// Edit a grant that has been submitted.
        /// Fields passed in with None will be ignored and not updated.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::edit_grant())]
        pub fn edit_grant(
            origin: OriginFor<T>,
            grant_id: GrantId,
            edited_milestones: Option<BoundedPMilestones<T>>,
            edited_approvers: Option<BoundedApprovers<T>>,
            // edited_ipfs: Option <[u8; 32]>,
            edited_currency_id: Option<CurrencyId>,
            edited_amount_requested: Option<BalanceOf<T>>,
            edited_treasury_origin: Option<TreasuryOrigin>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut grant = PendingGrants::<T>::get(grant_id).ok_or(Error::<T>::GrantNotFound)?;

            ensure!(!grant.is_cancelled, Error::<T>::GrantCancelled);
            ensure!(grant.submitter == who, Error::<T>::OnlySubmitterCanEdit);

            if let Some(milestones) = edited_milestones {
                let total_percentage = milestones.iter().fold(Percent::zero(), |acc, x| {
                    acc.saturating_add(x.percentage_to_unlock)
                });
                ensure!(total_percentage.is_one(), Error::<T>::MustSumTo100);
                grant.milestones = milestones;
            }
            if let Some(approvers) = edited_approvers {
                grant.approvers = approvers;
            }
            // if let Some(ipfs) = edited_ipfs {
            //     grant.ipfs_hash = ipfs;
            // }
            if let Some(currency_id) = edited_currency_id {
                grant.currency_id = currency_id;
            }
            if let Some(balance) = edited_amount_requested {
                grant.amount_requested = balance;
            }
            if let Some(t_origin) = edited_treasury_origin {
                grant.treasury_origin = t_origin;
            }

            PendingGrants::<T>::insert(grant_id, grant);
            Self::deposit_event(Event::<T>::GrantEdited { grant_id });

            Ok(().into())
        }

        /// Set the grant as cancelled
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_grant())]
        pub fn cancel_grant(
            origin: OriginFor<T>,
            grant_id: GrantId,
            as_authority: bool,
        ) -> DispatchResultWithPostInfo {
            let grant = PendingGrants::<T>::get(grant_id).ok_or(Error::<T>::GrantNotFound)?;

            if as_authority {
                <T as Config>::CancellingAuthority::ensure_origin(origin)?;
            } else {
                let who = ensure_signed(origin.clone())?;
                ensure!(grant.submitter == who, Error::<T>::OnlySubmitterCanEdit);
            }
            PendingGrants::<T>::mutate(grant_id, |grant| {
                if let Some(g) = grant {
                    g.is_cancelled = true;
                }
            });
            Self::deposit_event(Event::<T>::GrantCancelled { grant_id });
            Ok(().into())
        }

        /// Once you are completely happy with the grant details and are ready to submit to treasury
        /// You call this and it'll allow you to generate a project account id.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::convert_to_project())]
        pub fn convert_to_project(
            origin: OriginFor<T>,
            grant_id: GrantId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let grant = PendingGrants::<T>::get(grant_id).ok_or(Error::<T>::GrantNotFound)?;

            ensure!(grant.submitter == who, Error::<T>::OnlySubmitterCanEdit);
            ensure!(!grant.is_cancelled, Error::<T>::GrantCancelled);
            ensure!(!grant.is_converted, Error::<T>::AlreadyConverted);

            let mut contributions: BTreeMap<
                AccountIdOf<T>,
                Contribution<BalanceOf<T>, BlockNumberFor<T>>,
            > = BTreeMap::new();
            let _ = grant
                .approvers
                .iter()
                .map(|approver_id| {
                    contributions.insert(
                        approver_id.clone(),
                        Contribution {
                            value: grant.amount_requested / (grant.approvers.len() as u32).into(),
                            timestamp: frame_system::Pallet::<T>::block_number(),
                        },
                    )
                })
                .collect::<Vec<_>>();

            <T as Config>::IntoProposal::convert_to_proposal(
                grant.currency_id,
                contributions,
                grant_id,
                grant.submitter.clone(),
                grant
                    .milestones
                    .try_into()
                    .map_err(|_| Error::<T>::Overflow)?,
                FundingType::Grant(grant.treasury_origin),
            )
            .map_err(|_| Error::<T>::GrantConversionFailedGeneric)?;

            T::DepositHandler::return_deposit(grant.deposit_id)?;
            let _ = PendingGrants::<T>::mutate_exists(grant_id, |grant| {
                if let Some(g) = grant {
                    g.is_converted = true;
                }
                None::<T>
            });

            Ok(().into())
        }


        /// This is a hack for the demo, itll work but if we want to convert straight to a project
        /// it can be done ALOT more efficiently.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::convert_to_project() + <T as Config>::WeightInfo::submit_initial_grant())]
        pub fn create_and_convert(
            origin: OriginFor<T>,
            //ipfs_hash: [u8; 32],
            proposed_milestones: BoundedPMilestones<T>,
            assigned_approvers: BoundedApprovers<T>,
            currency_id: CurrencyId,
            amount_requested: BalanceOf<T>,
            treasury_origin: TreasuryOrigin,
            grant_id: GrantId
        ) -> DispatchResultWithPostInfo {
            Self::submit_initial_grant(
                origin.clone(),
                proposed_milestones,
                assigned_approvers,
                currency_id,
                amount_requested,
                treasury_origin,
                grant_id,
            )?;
            Self::convert_to_project(origin, grant_id)?;
            Ok(().into())
        }


        // TODO: runtime api to get the deposit address of the grants sovereign account.
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Grant<T: Config> {
        pub milestones: BoundedPMilestones<T>,
        pub submitter: AccountIdOf<T>,
        pub approvers: BoundedApprovers<T>,
        //pub ipfs_hash: [u8; 32],
        pub created_on: BlockNumberFor<T>,
        pub is_cancelled: bool,
        pub is_converted: bool,
        pub currency_id: CurrencyId,
        pub amount_requested: BalanceOf<T>,
        pub treasury_origin: TreasuryOrigin,
        pub deposit_id: DepositIdOf<T>,
    }
}
