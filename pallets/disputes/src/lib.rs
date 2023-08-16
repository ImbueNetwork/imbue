#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub mod traits;
pub mod weights;
pub use weights::*;
use sp_runtime::{DispatchError, traits::AtLeast32BitUnsigned};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use traits::DisputeRaiser;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type DisputeKey = u32;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		type DisputeKey: AtLeast32BitUnsigned;
		type MaxReasonLength: Get<u32>;
		type MaxJurySize: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn disputes)]
    pub type Disputes<T:> =
        StorageMap<_, Blake2_128Concat, T::DisputeKey, Dispute<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DisputeRaised {who: AccountIdOf<T>},
		DisputeVotedOn {who: AccountIdOf<T>},
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn vote_on_dispute(
			origin: OriginFor<T>,
			dispute_key: T::DisputeKey,
			is_yay: bool,
		) -> DispatchResult {
			// get dispute struct
			// ensure caller is part of the jury
			// mutate vote accordingly.
			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn force_cancel_dispute(
			origin: OriginFor<T>,
			dispute_key: T::DisputeKey,
			is_yay: bool,
		) -> DispatchResult {

			Ok(().into())
		}
	}

	impl<T: Config> DisputeRaiser<AccountIdOf<T>> for Pallet<T> {
		type DisputeKey = T::DisputeKey;
		fn raise_dispute(
			raised_by: AccountIdOf<T>,
			fund_account: AccountIdOf<T>,
			reason: Vec<u8>,
			project_id: u32,
			jury: Vec<AccountIdOf<T>>,
		) -> Result<(), DispatchError> {

			//incrementing the dispute_key 
			let dispute_key = DisputeKey::<T>::get().saturating_add(1);
			// creating the struct with the passed information and initializing vote as 0 initially
			let dispute:Dispute<T> = Dispute{
				raised_by,
				fund_account,
				votes: Vote{
					yay: 0,
					nay: 0,
					is_approved: false,
				},
				reason,
				jury,
			};

			//storing the raised dispute inside the disputes storage 
			Disputes::<T>::insert(dispute_key, dispute);

			// Raise Event 
			//TODO need to check if want to add more information while raising a dispute like returning the whole dispute struct
			Self::deposit_event(Event::DisputeRaised(
				raised_by
            ));

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
		jury: BoundedVec<AccountIdOf<T>, <T as Config>::MaxJurySize>
	}

	#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
   pub struct Vote {
    yay: u32 ,
    nay: u32,
    is_approved: bool,
}

}