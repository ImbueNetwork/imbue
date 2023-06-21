#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

pub mod traits;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use orml_traits::MultiReservableCurrency;
	use common_types::CurrencyId;

	type AccountIdOf<T> = <T as Config::frame_system>::AccountId;
	type SponsorOf<T> = AccountIdOf<T>;
	type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
		/// The authority appropriate to do call force extrinsics.
		type ForceAuthority: EnsureOrigin<AccountIdOf<Self>>;
		/// The handle used to initiate democracy calls.
		type DemocracyHandle: traits::DemocracyHandle<AccountIdOf<Self>>;
		/// The max number of candidates per wave.
		type MaxCandidatesPerShortlist: Get<u32>;
		/// The amount of time before a shortlist is moved to be voted on.
		type ShortlistPeriod: Get<BlockNumberFor<Self>>
		/// The minimum deposit required for a freelancer to hold fellowship status.
		type MembershipDeposit: Get<BalanceOf<Self>>;
		/// Deal with the unbalance when a freelancer gets their deposit slashed.
		type OnSlash: OnUnbalanced<BalanceOf<Self>>;
		/// The types of role one wants in the fellowship.
		type Role: Member
		+ TypeInfo
		+ Default
		+ MaxEncodedLen
		+ FullCodec
		+ FullEncode
		+ Copy;

		type Vetter: 
	}

	/// Used to map who is a part of the fellowship.
	/// Returns the role of the account
    #[pallet::storage]
    pub type Roles<T> =
        StorageMap<_, Blake2_128Concat, AccountIdOf<T>, T::Role, OptionQuery>;

	/// Contains the shortlist of candidates to be sent for approval.
	#[pallet::storage]
    pub type CandidateShortlist<T> =
        StorageValue<_, BoundedVec<AccountIdOf<T>, <T as Config>::MaxCandidatesPerShortlist>, ValueQuery>;

	/// Holds all the accounts that are able to become fellows that have not given their deposit for membership.
	#[pallet::storage]
	pub type PendingFellows<T> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, (), ValueQuery>;

	/// Keeps track of the deposits taken from a fellow. 
	/// Needed incase the reserve amount will change.
	#[pallet::storage]
	pub type FellowshipReserves<T> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, ValueQuery>;
	
	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		FellowshipAdded(AccountIdOf<T>),
		FellowshipRemoved(AccountIdOf<T>),
		MemberAddedToPendingFellows(AccountIdOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// This person does not have a role in the fellowship.
		NotAFellow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// An origin check wrapping the standard add_to_fellowship call.
		/// Force add someone to the fellowship. This is required to be called by the ForceOrigin
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn force_add_fellowship(origin: OriginFor<T>, who: AccountIdOf<T>, deposit_sponsor: AccountIdOf<T>) -> DispatchResult {
            <T as Config>::ForceAuthority::ensure_origin(origin)?;
			<Self as FellowshipHandler>::add_to_fellowship(&who, deposit_sponsor)?
			Self::deposit_event(Event::<T>::FellowshipAdded(who));
			Ok(().into())
		}

		/// Remove the account from the fellowship, 
		/// Called by the fellow and returns the deposit to them.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn leave_fellowship(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			<Self as FellowshipHandler>::remove(&who, deposit_sponsor)?
			Self::deposit_event(Event::<T>::FellowshipRemoved(who));
			Ok(().into())
		}

		/// Force remove a fellow and slashed their deposit as defined in the Config.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn force_remove_and_slash_fellowship(origin: OriginFor<T>) -> DispatchResult {
            <T as Config>::ForceAuthority::ensure_origin(origin)?;s
			<Self as FellowshipHandler>::remove(&who, deposit_sponsor)?

			todo!()
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn add_candidate_to_shortlist(origin: OriginFor<T>) -> DispatchResult {
			// Ensure that the candidate has enough imbue in the account to take the deposit.
			// Saves hassle for later
			todo!()
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn remove_candidate_from_shortlist(origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}
	}

	impl<T: crate::Config> FellowshipHandler<AccountIdOf<T>> for Pallet<T> {
		type Role = <T as Config>::Role;
		/// Add someone to the fellowship, if this fails to be tried again on demand.
		/// The usual reason this will fail is due to not having enough $IMBU.
		/// The deposit amount is defined in the Config.
		fn add_to_fellowship(who: &AccountIdOf<T>, role: Role) -> Result<(), DispatchError> {
			// If they aleady have a role then dont reserve as the reservation has already been taken.
			// This would only happen if a role was changed.
			if !Roles::<T>::contains_key(who) {
				let membership_deposit = <T as Config>::MembershipDeposit::get();
				if <T as Config>::MultiCurrency::can_reserve(CurrencyId::Native, who, membership_deposit) {
					<T as Config>::MultiCurrency::reserve(CurrencyId::Native, who, membership_deposit);
					FellowshipReserves::<T>::insert(who, membership_deposit);
					Roles::<T>::insert(who, role);
				} else {
					PendingFellows::<T>::insert(who, ());
					Self::deposit_event(Event::<T>::MemberAddedToPendingFellows(who.clone()));
				}
			} else {
				Roles::<T>::insert(who, role);
			}
		}
		/// Revoke the fellowship from an account.
		/// If they have not paid the deposit but are eligable then they can still be revoked
		/// using this method.
		fn revoke_fellowship(who: &AccountId, slash_deposit: bool) -> Result<(), DispatchError> {

		}
	}

	pub struct EnsureFellow(T);
	impl<T: Config> EnsureOrigin<AccountIdOf<T>> for EnsureFellowship {
		//todo!
	}
	
	pub struct EnsureVetter(T);
	impl<T: Config> EnsureOrigin<AccountIdOf<T>> for EnsureFellowship {
		//todo!
	}

 }


