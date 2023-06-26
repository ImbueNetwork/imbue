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
	use sp_runtime::traits::BadOrigin;

	type AccountIdOf<T> = <T as Config::frame_system>::AccountId;
	type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
	type ShortlistRound = u32;

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
		/// Currently just send all slash deposits to a single account.
		/// TODO: use OnUnbalanced.
		type SlashAccount: Get<AccountIdOf<Self>>;
	}

	/// Used to map who is a part of the fellowship.
	/// Returns the role of the account
    #[pallet::storage]
    pub type Roles<T> =
        StorageMap<_, Blake2_128Concat, AccountIdOf<T>, T::Role, OptionQuery>;

	/// Contains the shortlist of candidates to be sent for approval.
	#[pallet::storage]
    pub type CandidateShortlist<T> =
        StorageMap<_, ShortlistRound, BoundedVec<AccountIdOf<T>, <T as Config>::MaxCandidatesPerShortlist>, ValueQuery>;

	/// Keeps track of the round the shortlist is in.
	#[pallet::storage]
	pub type ShortlistRound<T> = StorageValue<_, ShortlistRound, ValueQuery>;

	#[pallet::storage]
	/// Holds all the accounts that are able to become fellows that have not given their deposit for membership.
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
		FellowshipSlashed(AccountIdOf<T>),
		MemberAddedToPendingFellows(AccountIdOf<T>),
		CandidateAddedToShortlist(AccountIfOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// This account does not have a role in the fellowship.
		RoleNotFound,
		/// This account is not a fellow.
		NotAFellow,
		/// This account is not a Vetter.
		NotAVetter,
		/// Already a fellow.
		AlreadyAFellow,
		/// The candidate must have the deposit amount to be put on the shortlst.
		CandidateDepositRequired,
		/// The candidate is already on the shortlist.
		CandidateAlreadyOnShortlist,
		/// The maximum number of candidates has been reached.
		TooManyCandidates,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// An origin check wrapping the standard add_to_fellowship call.
		/// Force add someone to the fellowship. This is required to be called by the ForceOrigin
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn force_add_fellowship(origin: OriginFor<T>, who: AccountIdOf<T>, deposit_sponsor: AccountIdOf<T>) -> DispatchResult {
            <T as Config>::ForceAuthority::ensure_origin(origin)?;
			<Self as FellowshipHandler>::add_to_fellowship(&who)?
			Self::deposit_event(Event::<T>::FellowshipAdded(who));
			Ok(().into())
		}

		/// Remove the account from the fellowship, 
		/// Called by the fellow and returns the deposit to them.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn leave_fellowship(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO: ensure that the fellow is not currently in a dispute.
			<Self as FellowshipHandler>::revoke_fellowship(&who, false)?
			Self::deposit_event(Event::<T>::FellowshipRemoved(who));
			Ok(().into())
		}

		/// Force remove a fellow and slashed their deposit as defined in the Config.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn force_remove_and_slash_fellowship(origin: OriginFor<T>) -> DispatchResult {
            <T as Config>::ForceAuthority::ensure_origin(origin)?;
			<Self as FellowshipHandler>::revoke_fellowship(&who, true)?;
			Self::deposit_event(Event::<T>::FellowshipSlashed(who));
			Ok(().into())
		}


		/// Add a candidate to a shortlist. 
		/// The caller must be of type Vetter to add to a shortlist.
		/// Also the candidate must already have the minimum deposit required.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn add_candidate_to_shortlist(origin: OriginFor<T>, candidate: AccountIdOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(EnsureFellowshipRole::<T>::ensure_has_role(&who).is_ok(), Error::<T>::NotAVetter);
			ensure!(Roles::<T>::get(&candidate).is_none(), Error::<T>::AlreadyAFellow);
			ensure(MultiCurrency::can_reserve(CurrencyId::Native, &candidate, <T as Config>::MembershipDeposit::get()), Error::<T>::CandidateDepositRequired);
			let _ = CandidateShortlist::<T>::try_mutate(ShortlistRound::<T>::get() |m_shortlist| -> DispatchResult {
				ensure!(!m_shortlist.contains_key(&candidate), Error::<T>::CandidateAlreadyOnShortlist);
				m_shortlist.try_insert(&candidate).map_err(|_| Error::<T>::TooManyCandidates)?;
			})?;

			Self::deposit_event(Event::<T>::CandidateAddedToShortlist(candidate));
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn remove_candidate_from_shortlist(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(EnsureFellowshipRole::<T>::ensure_role(&who, Role::Vetter).is_ok(), Error::<T>::NotAVetter);
		}
	}

	impl<T: crate::Config> FellowshipHandler<AccountIdOf<T>> for Pallet<T> {
		type Role = <T as Config>::Role;

		fn bulk_add_to_fellowship() -> Result<(), DispatchError>{
			// call add to fellowship with a limit.
			// remove all those from the CandidateShortlist
			// TODO: Candidate shortlist should have a map into shortlist version.
		}

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
			Ok(())
		}

		/// Revoke the fellowship from an account.
		/// If they have not paid the deposit but are eligable then they can still be revoked
		/// using this method.
		fn revoke_fellowship(who: &AccountId, slash_deposit: bool) -> Result<(), DispatchError> {
			let has_role = Roles::<T>::contains_key(who);
			ensure!(PendingFellows::<T>::contains_key(who) || has_role, NotAFellow);
			PendingFellows::<T>::remove(who);
			Roles::<T>::remove(who);

			let deposit_amount: BalanceOf<T> = <T as Config>::MembershipDeposit::get();
			// Essentially you can only slash a deposit if it has been taken
			// Deposits are only taken when a role is assigned
			if has_role {
				if slash_deposit {
					let _imbalance = <T as Config>::MultiCurrency::repatriate_reserved(
						CurrencyId::Native,
						who,
						&<T as Config>::SlashAccount::get(),
						deposit_amount,
						BalanceStatus::Free,
					)?;
				} else {
					<T as Config>::MultiCurrency::unreserve(CurrencyId::Native, who, deposit_amount);
				}
			}

			Ok(())
		}
	}

	/// Ensure that a account is of a given role.
	/// Used in other pallets like an ensure origin.
	pub struct EnsureFellowshipRole<T>(T);
	impl<T: Config> EnsureRole<AccountIdOf<T>, Role> for EnsureFellowshipRole<T> {
		type Success = ();
		
		fn ensure_role(acc: &AccountIdOf<T>, role: Role) -> Result<Self::Success, BadOrigin> {
			let actual = Roles::<T>::get(acc).ok_or(BadOrigin)?
			if role == actual {
				Ok(())
			} else {
				Err(Error::<T>::BadOrigin)
			}
		}

		fn ensure_has_role(acc: &AccountId) -> Result<Self::Success, BadOrigin> {
			let _ = Roles::<T>::get(acc).ok_or(BadOrigin);
			Ok(())
		}
	}

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub enum Role {
		Vetter, 
		Freelancer,
	}

 }




