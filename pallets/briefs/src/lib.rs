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

    type BriefHash = H256;

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
        type MaximumApplicants: Get<u32>;
        // The fee taken for submitting a brief could be a deposit?
        //type BriefSubmissionFee: Get<Percent>;
        /// Hasher used to generate brief hash
        type BriefHasher: Hasher;

        /// The amount of time applicants have to submit an application.
        type ApplicationSubmissionTime: Get<BlockNumberFor<Self>>;

        type AuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // The deposit required in IMBU to increase sybil resistance.
       // type AuctionDeposit: Get<BalanceOf<Self>>;
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

    /// The list of applications to a Brief, to be cleared only once the brief has been started.
    /// Key: BriefHash
    /// Value: List of applicants.
    #[pallet::storage]
    #[pallet::getter(fn brief_applications)]
    pub type BriefApplications<T> =
        StorageMap<_, Blake2_128Concat, BriefHash, BoundedApplications<T>, OptionQuery>;

    /// The list of accounts approved to apply for work. 
    /// Key: AccountId
    /// Value: Unit
    #[pallet::storage]
    #[pallet::getter(fn approved_accounts)]
    pub type ApprovedAccounts<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, (), ValueQuery>;

    /// Contains the briefs that are open for applicants.
    /// Key: BriefId.
    /// Value: Unit. 
    #[pallet::storage]
    pub type BriefsOpenForApplications<T> = StorageMap<_, Blake2_128Concat, BriefHash, (), ValueQuery>;

    /// Contains the briefs that are open for applicants.
    /// Key: BlockNumber the applications expire.
    /// Value: The list of briefs that are going to stop accepting applicants.
    #[pallet::storage]
    pub type BriefApplicationExpirations<T> = StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, BoundedBriefsPerBlock, OptionQuery>;
    
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        BriefSubmitted(BriefHash),
        ApplicationSubmitted(AccountIdOf<T>),
        ApplicationAccepted{brief: BriefHash, applicant: AccountIdOf<T>},
        AccountApproved(AccountIdOf<T>),

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
        /// You have already applied for this brief.
        AlreadyApplied,
        /// Brief already exists in the block, please don't submit duplicates.
        BriefAlreadyExists,
        /// Maximum Applications have been reached.
        MaximumApplicants,
        /// Brief not found.
        BriefNotFound,
        /// The BriefId generation failed.
        BriefHashingFailed,
        /// You do not have the authority to do this.
        NotAuthorised,
        /// the bounty required for this brief has not been met.
        BountyTotalNotMet,
        /// Current contribution exceeds the maximum total bounty.
        ExceedTotalBounty,
        /// You are not able to apply for this brief at this time.
        BriefClosedForApplications,
        /// There are too many briefs open for this block, try again later.
        BriefLimitReached,
        /// Currency must be set to add to a bounty.
        BriefCurrencyNotSet
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a brief to recieve applications.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn submit_brief_direct(
            origin: OriginFor<T>,
            ipfs_hash: BriefHash,
            bounty_total: BalanceOf<T>,
            initial_contribution: BalanceOf<T>,
            currency_id: CurrencyId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                initial_contribution >= <T as Config>::MinimumDeposit::get(),
                Error::<T>::DepositBelowMinimum
            );
            ensure!(
                bounty_total >= <T as Config>::MinimumBounty::get(),
                Error::<T>::BountyBelowMinimum
            );
            ensure!(
                bounty_total >= initial_contribution,
                Error::<T>::ContributionMoreThanBounty
            );

            // Malicious users can still submit briefs without an ipfs_hash (or an invalid one).
            // Therefore we must check that this item does exist in storage in an ocw and possible slash those who are malicious.
            // append_to_id_verification(&ipfs_hash);

            //let brief_id: BriefHash = BriefPreImage::<T>::generate_hash(&who, &bounty_total, &currency_id, off_chain_ref_id)?;

            // I am led to believe that we can use the ipfs hash as a unique identifier so long as we have a nonce contained within the data.
            // The main problem with this approach is that if the data changes, so does the hash.
            // Alas when updating the brief we must update ipfs, get the hash, and submit that atomically.
            ensure!(
                Briefs::<T>::get(ipfs_hash).is_none(),
                Error::<T>::BriefAlreadyExists
            );

            let new_brief = BriefData {
                created_by: who.clone(),
                bounty_total: Some(bounty_total), 
                current_contribution: Some(initial_contribution),
                currency_id: Some(currency_id),
                created_at: frame_system::Pallet::<T>::block_number(),
                is_auction: false,
            };

            <T as Config>::RMultiCurrency::reserve(currency_id, &who, initial_contribution)?;
            Briefs::<T>::insert(ipfs_hash, new_brief);
            let _ = Self::open_brief_for_applications(ipfs_hash);

            Self::deposit_event(Event::<T>::BriefSubmitted(ipfs_hash));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn submit_brief_auction(
            origin: OriginFor<T>,
            ipfs_hash: BriefHash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Take deposit? to increase sybil resistance.
            // Look at extrinsic submit_brief_direct for related comments
            ensure!(
                Briefs::<T>::get(ipfs_hash).is_none(),
                Error::<T>::BriefAlreadyExists
            );

            let new_brief = BriefData::new(who, None, None, None, frame_system::Pallet::<T>::block_number(), true);
            Briefs::<T>::insert(ipfs_hash, new_brief);
            let _ = Self::open_brief_for_applications(ipfs_hash)?;
            
            Self::deposit_event(Event::<T>::BriefSubmitted(ipfs_hash));

            Ok(())
        }

        /// Submit an application to a brief.
        /// Auctioning comes after the application process has closed.
        /// So there should not be any dealings with balances here.
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn submit_application(origin: OriginFor<T>, brief_id: BriefHash) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let is_approved = ApprovedAccounts::<T>::contains_key(&who);
            ensure!(is_approved, Error::<T>::OnlyApprovedAccountPermitted);

            let _ = Briefs::<T>::get(brief_id).ok_or(Error::<T>::BriefNotFound)?;
            let mut applicants: BoundedApplications<T> = BriefApplications::<T>::get(brief_id).unwrap_or(BTreeMap::new().try_into().expect("New map is smaller than bound; qed"));

            ensure!(applicants.get(&who).is_none(), Error::<T>::AlreadyApplied);
            ensure!(BriefsOpenForApplications::<T>::contains_key(brief_id), Error::<T>::BriefClosedForApplications);

            if applicants.try_insert(who.clone(), ()).is_ok() {
                BriefApplications::<T>::insert(brief_id, applicants);
            } else {
                return Err(Error::<T>::MaximumApplicants.into());
            };

            Self::deposit_event(Event::<T>::ApplicationSubmitted(who));
            Ok(())
        }



        /// todo: test
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn add_bounty(
            origin: OriginFor<T>,
            brief_id: BriefHash,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            // No check as to who is the signer?
            // If someone who isnt the brief owner sends the funds we have no record of the reservation.
            // Therefore when trying to send the reserved funds (when creating the proposal) the sum total will not be enough.
            // Either Keep a record of contributions, (like in proposals), or ensure that only the brief owner can contribute.

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

        /// Accept an application to a brief, 
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn accept_application(origin: OriginFor<T>, brief_id: BriefHash) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let brief = Briefs::<T>::get(brief_id).ok_or(Error::<T>::BriefNotFound)?;
            ensure!(brief.created_by == who, Error::<T>::NotAuthorised);
            ensure!(
                brief.bounty_total == brief.current_contribution,
                Error::<T>::BountyTotalNotMet
            );

            // todo:
            Self::deposit_event(Event::<T>::ApplicationAccepted{brief: brief_id, applicant: who});
            Ok(())
        }
        
        // todo: validation
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn approve_account(origin: OriginFor<T>, account_id: AccountIdOf<T>) -> DispatchResult {
            <T as Config>::AuthorityOrigin::ensure_origin(origin)?;
            ApprovedAccounts::<T>::insert(&account_id, ());
            Self::deposit_event(Event::<T>::AccountApproved(account_id));

            Ok(())
        }
    }

    #[pallet::hooks]
    impl <T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        // Get all the briefs that need to close their application status and close them.
        fn on_initialize(b: T::BlockNumber) -> Weight {
            let mut weight = Weight::default();
            weight += Self::close_briefs_for_applications(b);

            weight
        }
    }

    impl <T: Config> Pallet<T> {
        /// Keep track of wether the brief can still be applied to and when the brief application period closes.
        // Todo: test
        fn open_brief_for_applications(brief_id: BriefHash) -> Result<(), DispatchError> {
            let expiration_time = <T as Config>::ApplicationSubmissionTime::get() + frame_system::Pallet::<T>::block_number();
            let mut briefs_for_expiration = BriefApplicationExpirations::<T>::get(expiration_time).unwrap_or(vec![].try_into().expect("empty vec is less than bound; qed"));

            briefs_for_expiration.try_push(brief_id).map_err(|_| Error::<T>::BriefLimitReached)?;

            BriefsOpenForApplications::<T>::insert(brief_id, ());
            BriefApplicationExpirations::<T>::insert(expiration_time, briefs_for_expiration);

            Ok(())
        }

        // todo: test
        fn close_briefs_for_applications(block_number: T::BlockNumber) -> Weight {
            let mut weight = Weight::default();
            
            let briefs = BriefApplicationExpirations::<T>::get(block_number).unwrap_or(vec![].try_into().expect("Empty vec is less than bound; qed"));
            weight += T::DbWeight::get().reads(1);

            for brief_id in briefs {
                BriefsOpenForApplications::<T>::remove(brief_id);
                weight += T::DbWeight::get().reads_writes(1, 1);
            }

            BriefApplicationExpirations::<T>::remove(block_number);
            weight += T::DbWeight::get().reads_writes(1, 1);

            weight
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
        is_auction: bool,
    }

    impl<T: Config> BriefData<T> {
        pub fn new(created_by: AccountIdOf<T>, bounty_total: Option<BalanceOf<T>>, current_contribution: Option<BalanceOf<T>>, currency_id: Option<CurrencyId>, created_at: BlockNumberFor<T>, is_auction: bool) -> Self {
                Self {
                    created_at,
                    created_by,
                    bounty_total,
                    currency_id,
                    current_contribution,
                    is_auction,
                }
        }

    }

}



    // /// This is probably going to be removed.
    // #[derive(Encode, Hash)]
    // pub struct BriefPreImage<T: Config> {
    //     created_by: Vec<u8>,
    //     bounty_total: Vec<u8>,
    //     currency_id: Vec<u8>,
    //     // This must not be the ipfs hash as that will change with new content.
    //     // It can however be a field in the storage item.
    //     off_chain_ref_id: u32,
    //     phantom: PhantomData<T>,
    // }

    // impl<T: Config> BriefPreImage<T> {
    //     fn new<'a>(
    //         created_by: &'a AccountIdOf<T>,
    //         bounty_total: &'a BalanceOf<T>,
    //         currency_id: &'a CurrencyId,
    //         off_chain_ref_id: u32,
    //     ) -> Self {
    //         Self {
    //             created_by: <AccountIdOf<T> as Encode>::encode(created_by),
    //             bounty_total: <BalanceOf<T> as Encode>::encode(bounty_total),
    //             currency_id: <CurrencyId as Encode>::encode(currency_id),
    //             off_chain_ref_id,
    //             phantom: PhantomData,
    //         }
    //     }

    //     pub fn generate_hash<'a>(
    //         created_by: &'a AccountIdOf<T>,
    //         bounty_total: &'a BalanceOf<T>,
    //         currency_id: &'a CurrencyId,
    //         off_chain_ref_id: u32,
    //     ) -> Result<BriefHash, DispatchError> {
    //         let preimage: Self = Self::new(created_by, bounty_total, currency_id, off_chain_ref_id);
    //         let encoded = <BriefPreImage<T> as Encode>::encode(&preimage);
    //         let maybe_h256: Result<[u8; 32], _> =
    //             <<T as Config>::BriefHasher as Hasher>::hash(&encoded)
    //                 .as_ref()
    //                 .try_into();
    //         if let Ok(h256) = maybe_h256 {
    //             Ok(H256::from_slice(h256.as_slice()))
    //         } else {
    //             Err(Error::<T>::BriefHashingFailed.into())
    //         }
    //     }
    // }

    

