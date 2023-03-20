#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod traits;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

    use crate::traits::BriefEvolver;
    use common_types::CurrencyId;
    use frame_support::{pallet_prelude::*, traits::Get, sp_runtime::Saturating};
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use sp_core::{Hasher, H256};

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type BoundedBriefOwners<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxBriefOwners>;
    pub(crate) type Bounded
    pub(crate) type BriefHash = H256;
    pub(crate) type IpfsHash = H256;

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
        type BriefEvolver: BriefEvolver<AccountIdOf<Self>, BalanceOf<Self>, BlockNumberFor<Self>>;

        /// The maximum amount of owners to a brief.
        type MaxBriefOwners: Get<u32>;

        ///https://paritytech.github.io/substrate/master/frame_support/macro.defensive_assert.html
        //todo add defensive assert in runtime to ensure this is compatible with proposals
        /// Maximum milestones to a brief.
        type MaxMilestones: Get<u32>;
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
    pub type ApprovedAccounts<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, (), ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        BriefSubmitted(BriefHash),
        AccountApproved(AccountIdOf<T>),
        BriefEvolutionOccured(BriefHash),
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
        /// The brief hasnt yet been approved to commence by the freelancer.
        FreelancerApprovalRequired,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a bounty to a brief.
        /// A bounty must be fully contributed to before a piece of work is started.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn add_bounty(
            origin: OriginFor<T>,
            brief_id: BriefHash,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            // Only allow if its not an auction or it is an auction and the price has been set
            let who = ensure_signed(origin)?;
            let brief_record = Briefs::<T>::get(brief_id).ok_or(Error::<T>::BriefNotFound)?;
            ensure!(
                brief_record.brief_owners.contains(&who),
                Error::<T>::NotAuthorised
            );

            let new_amount: BalanceOf<T> = brief_record.current_contribution + amount;

            <T as Config>::RMultiCurrency::reserve(brief_record.currency_id, &who, amount)?;

            Briefs::<T>::mutate_exists(brief_id, |maybe_brief| {
                if let Some(brief) = maybe_brief {
                    brief.current_contribution = new_amount;
                }
            });

            Ok(())
        }

        /// Approve an account so that they can be accepted as an applicant.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn approve_account(origin: OriginFor<T>, account_id: AccountIdOf<T>) -> DispatchResult {
            <T as Config>::AuthorityOrigin::ensure_origin(origin)?;
            
            // Or if they are not voted by governance, be voted in by another approved freelancer? 
            // todo.

            ApprovedAccounts::<T>::insert(&account_id, ());
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
            bounty_total: BalanceOf<T>,
            initial_contribution: BalanceOf<T>,
            ipfs_hash: IpfsHash,
            currency_id: CurrencyId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if !brief_owners.contains(&who) {
                brief_owners
                    .try_push(who.clone())
                    .map_err(|_| Error::<T>::TooManyBriefOwners)?;
            }

            ensure!(
                ApprovedAccounts::<T>::contains_key(&applicant),
                Error::<T>::OnlyApprovedAccountPermitted
            );
            <T as Config>::RMultiCurrency::reserve(currency_id, &who, initial_contribution)?;

            // add breifs to OCW list to verify.
            
            let brief = BriefData::new(
                brief_owners,
                bounty_total,
                initial_contribution,
                currency_id,
                frame_system::Pallet::<T>::block_number(),
                ipfs_hash,
                applicant,
            );

            let brief_hash: BriefHash = brief.into_hash()?;
            Briefs::<T>::insert(brief_hash, brief);

            Self::deposit_event(Event::<T>::BriefSubmitted(brief_hash));

            Ok(())
        }

        /// Once the freelancer is happy with both the milestones and the offering this can be called.
        /// It will call the hook (if we want to use the hook) to bypass approval in the proposals pallet.
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn commence_work(origin: OriginFor<T>, brief_id: BriefHash) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let brief = Briefs::<T>::get(brief_id).ok_or(Error::<T>::BriefNotFound)?;

            ensure!(&who == &brief.applicant, Error::<T>::NotAuthorised);

            <T as Config>::BriefEvolver::convert_to_proposal(
                brief.brief_owners.to_vec(),
                brief.bounty_total,
                brief.currency_id,
                brief.current_contribution,
                brief.created_at,
                brief.ipfs_hash,
                brief.applicant,
            )
            .map_err(|_| Error::<T>::BriefConversionFailedGeneric)?;
            // todo, finer grained err handling
            Self::deposit_event(Event::<T>::BriefEvolutionOccured(brief_id));
            Ok(())
        }
    }

    /// The data assocaited with a Brief
    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct BriefData<T: Config> {
        brief_owners: BoundedBriefOwners<T>,
        bounty_total: BalanceOf<T>,
        currency_id: CurrencyId,
        current_contribution: BalanceOf<T>,
        created_at: BlockNumberFor<T>,
        ipfs_hash: IpfsHash,
        applicant: AccountIdOf<T>,
        // MILESTONES!!!          
    }

    impl<T: Config> Pallet<T> {
        pub fn get_remaining_bounty(brief_id: BriefHash) -> BalanceOf<T> {
            if let Some(brief) = Briefs::<T>::get(brief_id) {
                brief.bounty_total.saturating_sub(brief.current_contribution)
            } else {
                Default::default()
            }
        }
    }


    impl<T: Config> BriefData<T> {
        pub fn new(
            brief_owners: BoundedBriefOwners<T>,
            bounty_total: BalanceOf<T>,
            current_contribution: BalanceOf<T>,
            currency_id: CurrencyId,
            created_at: BlockNumberFor<T>,
            ipfs_hash: IpfsHash,
            applicant: AccountIdOf<T>,
        ) -> Self {
            Self {
                created_at,
                brief_owners,
                bounty_total,
                currency_id,
                current_contribution,
                ipfs_hash,
                applicant,
            }
        }

        pub fn into_hash(&self) -> Result<BriefHash, Error::<T>> {
            let preimage = BriefPreImage { 
                brief_owners: self.brief_owners.to_vec()
                    .iter()
                    .map(<AccountIdOf<T> as Encode>::encode)
                    .fold(vec![], |mut acc: Vec<u8>, mut n: Vec<u8>| {
                        acc.append(&mut n);
                        acc
                    }),
                bounty_total: <BalanceOf<T> as Encode>::encode(&self.bounty_total),
                currency_id: <CurrencyId as Encode>::encode(&self.currency_id),
                ipfs_hash: self.ipfs_hash.0.to_vec(),
                applicant: <AccountIdOf<T> as Encode>::encode(&self.applicant),
                phantom: PhantomData,
            };
            let encoded = <BriefPreImage<T> as Encode>::encode(&preimage);
            let maybe_h256: Result<[u8; 32], _> =
                <<T as Config>::BriefHasher as Hasher>::hash(&encoded)
                    .as_ref()
                    .try_into();
            if let Ok(h256) = maybe_h256 {
                Ok(H256::from_slice(h256.as_slice()))
            } else {
                Err(Error::<T>::BriefHashingFailed)
            }
        }
    }

    /// The preimage for the id of the brief in storage.
    #[derive(Encode, Hash)]
    pub struct BriefPreImage<T: Config> {
        brief_owners: Vec<u8>,
        bounty_total: Vec<u8>,
        currency_id: Vec<u8>,
        ipfs_hash: Vec<u8>,
        phantom: PhantomData<T>,
        applicant: Vec<u8>,
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct BriefMilestone<T: Config> {
        milestone_key: MilestoneKey,
        name: Vec<u8>,
        percentage_to_unlock: u32,
    }

}
