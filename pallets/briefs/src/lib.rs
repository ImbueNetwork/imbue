#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod traits;
use traits::BriefEvolver;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

    use common_types::CurrencyId;
    use frame_support::{pallet_prelude::*, traits::Get, BoundedBTreeMap};
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use sp_core::{Hasher, H256};
    use sp_std::collections::btree_map::BTreeMap;


    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> = <<T as Config>::RMultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    type BoundedApplications<T> =
        BoundedBTreeMap<AccountIdOf<T>, (), <T as Config>::MaximumApplicants>;
    type BoundedBriefsPerBlock = BoundedVec<BriefHash, ConstU32<100>>;
    type BoundedBriefOwners<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxBriefOwners>;

    type BriefHash = H256;
    type IpfsHash = [u8; 32];

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type RMultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        /// The minimum deposit required to submit a brief
        // SHOULD THIS BE AS A PERCENT OF BOUNTY? TODO:.
        type MinimumDeposit: Get<BalanceOf<Self>>;
        /// The minimum bounty required to submit a brief.
        type MinimumBounty: Get<BalanceOf<Self>>;
        /// Maximum amount of applicants to a brief.
        type BriefHasher: Hasher;

        type AuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The type that allows for evolution from brief to proposal.
        type BriefEvolver: BriefEvolver<BalanceOf<Self>, AccountIdOf<Self>, BlockNumberFor<Self>>;

        /// The maximum amount of owners to a brief
        type MaxBriefOwners: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn briefs)]
    pub type Briefs<T> = CountedStorageMap<
        _,
        Blake2_128Concat,
        BriefHash,
        BriefData<T>,
        OptionQuery,
    >;

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
        BriefEvolutionOccured(BriefHash)
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
        BriefCurrencyNotSet
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
            let mut brief_record = Briefs::<T>::get(&brief_id).ok_or(Error::<T>::BriefNotFound)?;

            let new_amount: BalanceOf<T> = brief_record.current_contribution.unwrap_or(Default::default()) + amount;
            let currency_id = brief_record.currency_id.ok_or(Error::<T>::BriefCurrencyNotSet)?;

            ensure!(
                brief_record.bounty_total.ok_or(Error::<T>::BriefCurrencyNotSet)? >= new_amount,
                Error::<T>::ExceedTotalBounty
            );

            brief_record.current_contribution = Some(new_amount);
            <T as Config>::RMultiCurrency::reserve(currency_id, &who, amount)?;

            Briefs::<T>::mutate_exists(&brief_id, |brief| {
                *brief = Some(brief_record);
            });

            Ok(())
        }


        /// Approve an account so that they can be accepted as an applicant.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn approve_account(origin: OriginFor<T>, account_id: AccountIdOf<T>) -> DispatchResult {
            <T as Config>::AuthorityOrigin::ensure_origin(origin)?;
            ApprovedAccounts::<T>::insert(&account_id, ());
            Self::deposit_event(Event::<T>::AccountApproved(account_id));

            Ok(())
        }

        /// Create a brief to be fully funded.
        /// In the current state the applicant must be approved.
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn create_brief(origin: OriginFor<T>, brief_owners: BoundedBriefOwners<T> , applicant: AccountIdOf<T>, bounty_total: BalanceOf<T>, initial_contribution: BalanceOf<T>, ipfs_hash: IpfsHash, currency_id: CurrencyId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(ApprovedAccounts::<T>::contains_key(&applicant), Error::<T>::OnlyApprovedAccountPermitted);
            <T as Config>::RMultiCurrency::reserve(currency_id, &who, initial_contribution)?;

            // add breifs to OCW list to verify.
            let brief_hash = BriefPreImage::generate_hash(&brief_owners.to_vec(), &bounty_total, &currency_id, &ipfs_hash);
            let brief = BriefData::new(who.clone(), bounty_total, initial_contribution, currency_id, frame_system::Pallet::<T>::block_number(), ipfs_hash);

            Briefs::<T>::insert(&brief_hash, brief);

            Self::deposit_event(Event::<T>::BriefSubmitted(brief_hash));

            Ok(())
        }
    }


    }

    #[pallet::hooks]
    impl <T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        // Get all the briefs that need to close their application status and close them.
        fn on_initialize(b: T::BlockNumber) -> Weight {
            Weight::default()
        }
    }


    /// The data assocaited with a Brief
    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
    pub struct BriefData<T: Config> {
        created_by: AccountIdOf<T>,
        bounty_total: Option<BalanceOf<T>>,
        currency_id: Option<CurrencyId>,
        current_contribution: Option<BalanceOf<T>>,
        created_at: BlockNumberFor<T>,
        ipfs_hash: IpfsHash,
    }

    impl<T: Config> BriefData<T> {
        pub fn new(created_by: AccountIdOf<T>, bounty_total: Option<BalanceOf<T>>, current_contribution: Option<BalanceOf<T>>, currency_id: Option<CurrencyId>, created_at: BlockNumberFor<T>, ipfs_hash: IpfsHash) -> Self {
                Self {
                    created_at,
                    created_by,
                    bounty_total,
                    currency_id,
                    current_contribution,
                    ipfs_hash,
                }
        }

    }



     /// This is probably going to be removed.
     #[derive(Encode, Hash)]
     pub struct BriefPreImage<T: Config> {
         brief_owners: Vec<u8>,
         bounty_total: Vec<u8>,
         currency_id: Vec<u8>,
         // This must not be the ipfs hash as that will change with new content.
         // It can however be a field in the storage item.
         ipfs_hash: IpfsHash,
         phantom: PhantomData<T>,
     }

     impl<T: Config> BriefPreImage<T> {
         pub fn generate_hash<'a>(
            brief_owners: &'a Vec<AccountIdOf<T>>,
            bounty_total: &'a BalanceOf<T>,
            currency_id: &'a CurrencyId,
            ipfs_hash: u32,
         ) -> Result<BriefHash, DispatchError> {
            let preimage = Self {
                brief_owners:  brief_owners.iter().map(|acc| {
                    <AccountIdOf<T> as Encode>::encode(acc)
                }).collect::<Vec<Vec<u8>>>().into_flattened(),
                bounty_total: <BalanceOf<T> as Encode>::encode(bounty_total),
                currency_id: <CurrencyId as Encode>::encode(currency_id),
                ipfs_hash,
                phantom: PhantomData,
            }
             let encoded = <BriefPreImage<T> as Encode>::encode(&preimage);
             let maybe_h256: Result<[u8; 32], _> =
                 <<T as Config>::BriefHasher as Hasher>::hash(&encoded)
                     .as_ref()
                     .try_into();
             if let Ok(h256) = maybe_h256 {
                 Ok(H256::from_slice(h256.as_slice()))
             } else {
                 Err(Error::<T>::BriefHashingFailed.into())
             }
         }
     }

    

