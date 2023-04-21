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
    use frame_support::{pallet_prelude::*, BoundedVec};
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};

    use common_types::{milestone_origin::FundingType, CurrencyId, TreasuryOrigin};

    use pallet_proposals::{traits::IntoProposal, Contribution, ProposedMilestone};
    use sp_core::H256;
    use sp_std::collections::btree_map::BTreeMap;

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

    type BoundedPMilestones<T> =
        BoundedVec<ProposedMilestoneWithInfo, <T as Config>::MaxMilestonesPerGrant>;
    type BoundedApprovers<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxApprovers>;
    type BoundedGrantsSubmitted = BoundedVec<GrantId, ConstU32<500>>;
    type GrantId = H256;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Maximum amount of milestones per grants.
        type MaxMilestonesPerGrant: Get<u32>;
        /// The maximum approvers for a given grants.
        type MaxApprovers: Get<u32>;
        type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;

        /// The type that converts into a proposal for milestone submission.
        type IntoProposal: IntoProposal<
            AccountIdOf<Self>,
            BalanceOf<Self>,
            BlockNumberFor<Self>,
            <Self as pallet_timestamp::Config>::Moment,
        >;
        /// The authority allowed to cancel a pending grants.
        type CancellingAuthority: EnsureOrigin<Self::RuntimeOrigin>;
    }

    /// Stores all the Grants waiting for approval, funding and eventual conversion into milestones.
    /// Key 1: GrantId
    /// Value: Grant<T>
    #[pallet::storage]
    pub type PendingGrants<T: Config> = StorageMap<_, Blake2_128, GrantId, Grant<T>, OptionQuery>;

    /// Stores all the grants a user has submitted.
    /// Key 1: AccountId
    /// Key 2: GrantId
    /// Value: ()
    #[pallet::storage]
    pub type GrantsSubmittedBy<T: Config> =
        StorageDoubleMap<_, Blake2_128, AccountIdOf<T>, Blake2_128, GrantId, (), ValueQuery>;

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
        /// The grants already exists.
        GrantAlreadyExists,
        /// Overflow Error in pallet-grants.
        Overflow,
        /// Only the submitter can edit this grants.
        OnlySubmitterCanEdit,
        /// Cannot use a cancelled grants.
        GrantCancelled,
        /// This grants has already been converted.
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
        /// A grants starts here with nothing agreed upon and
        /// probably awaiting much back and forth.
        #[pallet::call_index(0)]
        #[pallet::weight(100_000)]
        pub fn submit_initial_grant(
            origin: OriginFor<T>,
            ipfs_hash: [u8; 32],
            proposed_milestones: BoundedPMilestones<T>,
            assigned_approvers: BoundedApprovers<T>,
            currency_id: CurrencyId,
            amount_requested: BalanceOf<T>,
            treasury_origin: TreasuryOrigin,
        ) -> DispatchResultWithPostInfo {
            let submitter = ensure_signed(origin)?;
            let total_percentage = proposed_milestones
                .iter()
                .fold(0u32, |acc, x| acc.saturating_add(x.percent.into()));
            ensure!(total_percentage == 100, Error::<T>::MustSumTo100);

            // TODO: Ensure that the approvers are in a select group??
            // TODO: take deposit to prevent spam? how else can we prevent spam
            // TODO: GENERATE grant_id. properly. or get as param
            let grant_id: GrantId = Default::default();
            ensure!(
                !PendingGrants::<T>::contains_key(grant_id),
                Error::<T>::GrantAlreadyExists
            );

            let grant = Grant {
                milestones: proposed_milestones,
                submitter: submitter.clone(),
                approvers: assigned_approvers,
                ipfs_hash,
                created_on: frame_system::Pallet::<T>::block_number(),
                is_cancelled: false,
                is_converted: false,
                currency_id,
                amount_requested,
                treasury_origin,
            };

            PendingGrants::<T>::insert(&grant_id, grant);
            GrantsSubmittedBy::<T>::insert(&submitter, &grant_id, ());

            Self::deposit_event(Event::<T>::GrantSubmitted {
                submitter,
                grant_id,
            });
            Ok(().into())
        }

        /// Edit a grants that has been submitted.
        /// Fields passed in with None will be ignored and not updated.
        #[pallet::call_index(1)]
        #[pallet::weight(100_000)]
        pub fn edit_grant(
            origin: OriginFor<T>,
            grant_id: GrantId,
            edited_milestones: Option<BoundedPMilestones<T>>,
            edited_approvers: Option<BoundedApprovers<T>>,
            edited_ipfs: Option<[u8; 32]>,
            edited_currency_id: Option<CurrencyId>,
            edited_amount_requested: Option<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut grant = PendingGrants::<T>::get(grant_id).ok_or(Error::<T>::GrantNotFound)?;

            ensure!(!grant.is_cancelled, Error::<T>::GrantCancelled);
            ensure!(&grant.submitter == &who, Error::<T>::OnlySubmitterCanEdit);

            if let Some(milestones) = edited_milestones {
                grant.milestones = milestones;
            }
            if let Some(approvers) = edited_approvers {
                grant.approvers = approvers;
            }
            if let Some(ipfs) = edited_ipfs {
                grant.ipfs_hash = ipfs;
            }
            if let Some(currency_id) = edited_currency_id {
                grant.currency_id = currency_id;
            }
            if let Some(balance) = edited_amount_requested {
                grant.amount_requested = balance;
            }

            PendingGrants::<T>::insert(&grant_id, grant);
            Self::deposit_event(Event::<T>::GrantEdited { grant_id });

            Ok(().into())
        }

        /// Set the grants as cancelled
        #[pallet::call_index(2)]
        #[pallet::weight(100_000)]
        pub fn cancel_grant(
            origin: OriginFor<T>,
            grant_id: GrantId,
            as_authority: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            let mut grant = PendingGrants::<T>::get(&grant_id).ok_or(Error::<T>::GrantNotFound)?;
            if as_authority {
                <T as Config>::CancellingAuthority::ensure_origin(origin)?;
            } else {
                ensure!(grant.submitter == who, Error::<T>::OnlySubmitterCanEdit);
            }

            grant.is_cancelled = true;
            PendingGrants::<T>::insert(&grant_id, grant);
            Self::deposit_event(Event::<T>::GrantCancelled { grant_id });

            Ok(().into())
        }

        /// Once you are completely happy with the grants details and are ready to submit to treasury
        /// You call this and itll allow you to generate a project account id.
        #[pallet::call_index(3)]
        #[pallet::weight(100_000)]
        pub fn convert_to_milestones(
            origin: OriginFor<T>,
            grant_id: GrantId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let grant = PendingGrants::<T>::get(grant_id).ok_or(Error::<T>::GrantNotFound)?;

            ensure!(&grant.submitter == &who, Error::<T>::OnlySubmitterCanEdit);
            ensure!(!grant.is_cancelled, Error::<T>::GrantCancelled);
            ensure!(!grant.is_converted, Error::<T>::AlreadyConverted);

            let mut contributions: BTreeMap<
                AccountIdOf<T>,
                Contribution<BalanceOf<T>, <T as pallet_timestamp::Config>::Moment>,
            > = BTreeMap::new();
            let _ = grant
                .approvers
                .iter()
                .map(|approver_id| {
                    contributions.insert(
                        approver_id.clone(),
                        Contribution {
                            value: grant.amount_requested / (grant.approvers.len() as u32).into(),
                            timestamp: pallet_timestamp::Pallet::<T>::get(),
                        },
                    )
                })
                .collect::<Vec<_>>();

            // TODO: fix this
            // For now we have to do a conversion into a simpler proposed milestone as pallet_proposals does not support ipfs data for them.
            let standard_proposed_ms = grant
                .milestones
                .iter()
                .map(|ms| ProposedMilestone {
                    percentage_to_unlock: ms.percent as u32,
                })
                .collect::<Vec<ProposedMilestone>>();

            let _ = <T as Config>::IntoProposal::convert_to_proposal(
                grant.currency_id,
                contributions,
                grant_id,
                grant.submitter,
                standard_proposed_ms,
                FundingType::Treasury(grant.treasury_origin),
            )
            .map_err(|_| Error::<T>::GrantConversionFailedGeneric)?;

            Ok(().into())
        }

        // TODO: runtime api to get the deposit address of the grants sovereign account.
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Grant<T: Config> {
        milestones: BoundedPMilestones<T>,
        submitter: AccountIdOf<T>,
        approvers: BoundedApprovers<T>,
        ipfs_hash: [u8; 32],
        created_on: BlockNumberFor<T>,
        is_cancelled: bool,
        is_converted: bool,
        currency_id: CurrencyId,
        amount_requested: BalanceOf<T>,
        treasury_origin: TreasuryOrigin,
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    pub struct ProposedMilestoneWithInfo {
        percent: u8,
        ipfs_hash: [u8; 32],
    }
}
