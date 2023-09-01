#![cfg_attr(not(feature = "std"), no_std)]

//FELIX REVIEW: Eventually it will be nice to have a short introduction here explaining what this pallet does and the
// avaliable methods etc.

pub use pallet::*;
pub mod traits;
use sp_runtime::traits::Saturating;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use codec::{FullCodec, FullEncode};
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchError};
    use traits::DisputeRaiser;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[pallet::config]
    //FELIX Review: Comment each of the config items so we know exactly what they are doing.
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        //Felix teachings --> to keep an associated type into storage it needs to implement certain traits
        type DisputeKey: AtLeast32BitUnsigned + FullEncode + FullCodec + MaxEncodedLen + TypeInfo;
        type MaxReasonLength: Get<u32>;
        type MaxJurySize: Get<u32>;
        type DisputeHooks: traits::DisputeHooks<Self::DisputeKey>;
        type TimeLimit: Get<<Self as frame_system::Config>::BlockNumber>;
        // type AuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    //Used to store the disputes that is being raised, given the dispute key it returns the Dispute
    #[pallet::storage]
    #[pallet::getter(fn disputes)]
    pub type Disputes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::DisputeKey, Dispute<T>, OptionQuery>;

    #[pallet::event]
    // FELIX REVIEW: the below generate_deposit line is depricated in the 9.0.43 so you can remove it completely.
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        //This event is emitted whenever a dispute has been successfully raised
        DisputeRaised { who: AccountIdOf<T> },
        // This event is emitted whenever there has been a voting successfully happened for a given dispute by
        // the authorized jury member
        DisputeVotedOn { who: AccountIdOf<T> },
    }


    #[pallet::error]
    pub enum Error<T> {
        //When there is no value present
        NoneValue,
        //Whenever we try to insert the storage beyond its bounded capacity
        StorageOverflow,
        //This error is thrown whenever the dispute key passed doesn't correspond to any dispute
        DisputeDoesNotExist,
        DisputeAlreadyExists,
        //wrong jury trying to vote
        InvalidJuryAccount,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        // FELIX: BENCHMARK
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn vote_on_dispute(
            origin: OriginFor<T>,
            dispute_key: T::DisputeKey,
            is_yay: bool,
        ) -> DispatchResult {
            // get dispute struct
            // ensure caller is part of the jury
            // mutate vote accordingly.            
            let who = ensure_signed(origin)?;

            //iterate over the disputes and update the voting state based on the passed vote
            Disputes::<T>::try_mutate(dispute_key, |dispute| {
                if let Some(d) = dispute {
                    ensure!(
                        d.jury.iter().any(|e| e == &who),
                        Error::<T>::InvalidJuryAccount
                    );
                    let vote = d.votes;
                    if is_yay {
                        vote.yay = vote.yay.saturating_add(1);
                    } else {
                        vote.nay = vote.nay.saturating_add(1);
                    }
                    Ok(())
                }
                else {
                    Err(Error::<T>::DisputeDoesNotExist)
                }
            });

            // //updating the votes
            // dispute.votes = vote;
            // //updated the dispute
            // Disputes::insert(dispute_key, dispute);

            Self::deposit_event(Event::DisputeVotedOn(who));
            Ok(().into())
        }

        #[pallet::call_index(1)]
        // FELIX REVIEW: Benchmarks
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn force_cancel_dispute(
            origin: OriginFor<T>,
            dispute_key: T::DisputeKey,
            is_yay: bool,
        ) -> DispatchResult {
            // FELIX: Ensure cancelling authority
            // remove
            // call hook on_cancel for trait use T::DisputeHooks
            // emit event
            Ok(().into())
        }
    }

    impl<T: Config> DisputeRaiser<AccountIdOf<T>> for Pallet<T> {
        type DisputeKey = T::DisputeKey;

        fn raise_dispute(
            dispute_key: Self::DisputeKey,
            raised_by: AccountIdOf<T>,
            fund_account: AccountIdOf<T>,
            reason: Vec<u8>,
            project_id: u32,
            jury: Vec<AccountIdOf<T>>,
        ) -> Result<(), DispatchError> {
            // creating the struct with the passed information and initializing vote as 0 initially
            let dispute: Dispute<T> = Dispute {
                raised_by,
                fund_account,
                votes: Vote { yay: 0, nay: 0 },
                reason,
                jury,
            };

            ensure!(
                !Disputes::<T>::contains_key(dispute_key),
                Error::<T>::DisputeAlreadyExists
            );
            //storing the raised dispute inside the disputes storage
            Disputes::<T>::insert(dispute_key, dispute);

            // Raise Event
            //SHANKAR if want to add more information while raising a dispute like returning the whole dispute struct to
            //get some more info about what has been raised or so on?
            Self::deposit_event(Event::DisputeRaised(raised_by));

            Ok(())
        }
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    struct Dispute<T: Config> {
        raised_by: AccountIdOf<T>,
        fund_account: AccountIdOf<T>,
        // TODO: Add balance type
        // currencyid: CurrencyId
        //fund_amount: BalanceOf<T>
        votes: Vote,
        reason: BoundedVec<u8, <T as Config>::MaxReasonLength>,
        jury: BoundedVec<AccountIdOf<T>, <T as Config>::MaxJurySize>,
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
    pub struct Vote {
        yay: u32,
        nay: u32,
    }

    enum Outcome {
        Refund,
        Continue,
        Slash,
    }
}
