#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

pub mod traits;
pub mod impls;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, BoundedBTreeMap};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiReservableCurrency, MultiCurrency, BalanceStatus};
	use common_types::CurrencyId;
	use sp_runtime::traits::{Zero};
	use sp_std::{convert::TryInto, vec};

	use crate::traits::{EnsureRole, FellowshipHandle, DemocracyHandle};
	use crate::impls::EnsureFellowshipRole;
	use crate::weights::WeightInfo;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type VetterIdOf<T> = AccountIdOf<T>; 
	type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
	type ShortlistRoundKey = u32;
	type BoundedShortlistPlaces<T> = BoundedBTreeMap<AccountIdOf<T>, (Role, Option<VetterIdOf<T>>), <T as Config>::MaxCandidatesPerShortlist>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
		/// The authority appropriate to do call force extrinsics.
		type ForceAuthority: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
		/// The handle used to initiate democracy calls.
		type DemocracyHandle: DemocracyHandle<AccountIdOf<Self>>;
		/// The max number of candidates per wave.
		type MaxCandidatesPerShortlist: Get<u32>;
		/// The amount of time before a shortlist is moved to be voted on.
		type ShortlistPeriod: Get<BlockNumberFor<Self>>;
		/// The minimum deposit required for a freelancer to hold fellowship status.
		type MembershipDeposit: Get<BalanceOf<Self>>;
		/// Currently just send all slash deposits to a single account.
		/// TODO: use OnUnbalanced.
		type SlashAccount: Get<AccountIdOf<Self>>;
        type WeightInfo: WeightInfo;
	}

	/// Used to map who is a part of the fellowship.
	/// Returns the role of the account
    #[pallet::storage]
    pub type Roles<T> =
        StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Role, OptionQuery>;

	/// Contains the shortlist of candidates to be sent for approval.
	#[pallet::storage]
    pub type CandidateShortlist<T> =
        StorageMap<_,  Blake2_128Concat, ShortlistRoundKey, BoundedShortlistPlaces<T>, ValueQuery>;

	/// Keeps track of the round the shortlist is in.
	#[pallet::storage]
	pub type ShortlistRound<T> = StorageValue<_, ShortlistRoundKey, ValueQuery>;

	#[pallet::storage]
	/// Holds all the accounts that are able to become fellows that have not given their deposit for membership.
	pub type PendingFellows<T> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Role, OptionQuery>;

	/// Keeps track of the deposits taken from a fellow. 
	/// Needed incase the reserve amount will change.
	#[pallet::storage]
	pub type FellowshipReserves<T> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, OptionQuery>;

	/// Keeps track of the accounts a fellow has recruited.
	/// Can be used to pay out completion fees.
	#[pallet::storage]
	pub type FellowToVetter<T> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, VetterIdOf<T>, OptionQuery>;
	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		FellowshipAdded(AccountIdOf<T>),
		FellowshipRemoved(AccountIdOf<T>),
		FellowshipSlashed(AccountIdOf<T>),
		MemberAddedToPendingFellows(AccountIdOf<T>),
		CandidateAddedToShortlist(AccountIdOf<T>),
		CandidateRemovedFromShortlist(AccountIdOf<T>),
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
		/// The fellowship deposit has could not be found, contact development.
		FellowshipReserveDisapeared,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let mut weight = Weight::default();
			if n % T::ShortlistPeriod::get() == Zero::zero() {
				let round_key = ShortlistRound::<T>::get();
				let shortlist =  CandidateShortlist::<T>::get(round_key);
				// TODO: benchmark add_to_fellowship and add the according weight.
				shortlist.iter().for_each(|(acc, (role, maybe_vetter))| {
					weight.saturating_add(T::WeightInfo::add_to_fellowship());
					Self::add_to_fellowship(acc, *role, maybe_vetter.as_ref());
				});

				CandidateShortlist::<T>::remove(round_key);
				ShortlistRound::<T>::put(round_key.saturating_add(1));
			}
			weight
        }
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An origin check wrapping the standard add_to_fellowship call.
		/// Force add someone to the fellowship. This is required to be called by the ForceOrigin
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn force_add_fellowship(origin: OriginFor<T>, who: AccountIdOf<T>, role: Role) -> DispatchResult {
            <T as Config>::ForceAuthority::ensure_origin(origin)?;
			<Self as FellowshipHandle<AccountIdOf<T>>>::add_to_fellowship(&who, role, None)?;
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
			<Self as FellowshipHandle<AccountIdOf<T>>>::revoke_fellowship(&who, false)?;
			Self::deposit_event(Event::<T>::FellowshipRemoved(who));
			Ok(().into())
		}

		/// Force remove a fellow and slashed their deposit as defined in the Config.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn force_remove_and_slash_fellowship(origin: OriginFor<T>, who: AccountIdOf<T>) -> DispatchResult {
            <T as Config>::ForceAuthority::ensure_origin(origin)?;
			<Self as FellowshipHandle<AccountIdOf<T>>>::revoke_fellowship(&who, true)?;
			Self::deposit_event(Event::<T>::FellowshipSlashed(who));
			Ok(().into())
		}

		/// Add a candidate to a shortlist. 
		/// The caller must be of type Vetter or Freelancer to add to a shortlist.
		/// Also the candidate must already have the minimum deposit required.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn add_candidate_to_shortlist(origin: OriginFor<T>, candidate: AccountIdOf<T>, role: Role) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(EnsureFellowshipRole::<T>::ensure_role_in(&who, vec![Role::Freelancer, Role::Vetter]).is_ok(), Error::<T>::NotAVetter);
			ensure!(Roles::<T>::get(&candidate).is_none(), Error::<T>::AlreadyAFellow);
			ensure!(T::MultiCurrency::can_reserve(CurrencyId::Native, &candidate, <T as Config>::MembershipDeposit::get()), Error::<T>::CandidateDepositRequired);
			let _ = CandidateShortlist::<T>::try_mutate(ShortlistRound::<T>::get(), |m_shortlist| {
				ensure!(!m_shortlist.contains_key(&candidate), Error::<T>::CandidateAlreadyOnShortlist);
				m_shortlist.try_insert(candidate.clone(), (role, Some(who))).map_err(|_| Error::<T>::TooManyCandidates)?;
				Ok::<(), DispatchError>(())
			})?;

			Self::deposit_event(Event::<T>::CandidateAddedToShortlist(candidate));
			Ok(())
		}

		/// Remove a candidate from the shortlist.
		/// The caller must have a role of either Vetter or Freelancer.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn remove_candidate_from_shortlist(origin: OriginFor<T>, candidate: AccountIdOf<T>) -> DispatchResult {
				let who = ensure_signed(origin)?;
			ensure!(EnsureFellowshipRole::<T>::ensure_role_in(&who, vec![Role::Freelancer, Role::Vetter]).is_ok(), Error::<T>::NotAVetter);
			let _ = CandidateShortlist::<T>::try_mutate(ShortlistRound::<T>::get(), |m_shortlist| {
				m_shortlist.remove(&candidate);
				Ok::<(), DispatchError>(())
			})?;

			Self::deposit_event(Event::<T>::CandidateRemovedFromShortlist(candidate));
			Ok(().into())			
		}

		/// If the freelancer fails to have enough native token at the time of shortlist approval they are
		/// added to the PendingFellows, calling this allows them to attempt to take the deposit and
		/// become a fellow.
		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn pay_deposit_to_remove_pending_status(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let role = PendingFellows::<T>::get(&who).ok_or(Error::<T>::NotAFellow)?;
			let membership_deposit = <T as Config>::MembershipDeposit::get();

			<T as Config>::MultiCurrency::reserve(CurrencyId::Native, &who, membership_deposit)?;
			FellowshipReserves::<T>::insert(&who, membership_deposit);
			PendingFellows::<T>::remove(&who);
			Roles::<T>::insert(&who, role);

			Self::deposit_event(Event::<T>::FellowshipAdded(who));
			Ok(().into())			
		}
	}

	impl<T: crate::Config> FellowshipHandle<AccountIdOf<T>> for Pallet<T> {
		type Role = crate::pallet::Role;

		/// Does no check on the Origin of the call.
		/// Add someone to the fellowship the only way this "fails" is when the candidate does not have
		/// enough native token for the deposit, this candidate is then added to PendingFellows where they
		/// can pay the deposit later to accept the membership.
		/// The deposit amount is defined in the Config.
		fn add_to_fellowship(who: &AccountIdOf<T>, role: Role, vetter: Option<&VetterIdOf<T>>) -> Result<(), DispatchError> {
			// If they aleady have a role then dont reserve as the reservation has already been taken.
			// This would only happen if a role was changed.
			if !Roles::<T>::contains_key(who) {
				let membership_deposit = <T as Config>::MembershipDeposit::get();
				if let Ok(_) = <T as Config>::MultiCurrency::reserve(CurrencyId::Native, who, membership_deposit) {
					FellowshipReserves::<T>::insert(who, membership_deposit);
					Roles::<T>::insert(who, role);
				} else {
					PendingFellows::<T>::insert(who, role);
					Self::deposit_event(Event::<T>::MemberAddedToPendingFellows(who.clone()));
				}
				if let Some(v) = vetter {
					FellowToVetter::<T>::insert(who, v);
				}
			} else {
				Roles::<T>::insert(who, role);
			}

			Ok(())
		}

		/// Does no check on the Origin of the call.
		/// Revoke the fellowship from an account.
		/// If they have not paid the deposit but are eligable then they can still be revoked
		/// using this method.
		fn revoke_fellowship(who: &AccountIdOf<T>, slash_deposit: bool) -> Result<(), DispatchError> {
			let has_role = Roles::<T>::contains_key(who);
			ensure!(PendingFellows::<T>::contains_key(who) || has_role, Error::<T>::NotAFellow);
			PendingFellows::<T>::remove(who);
			Roles::<T>::remove(who);
			FellowToVetter::<T>::remove(who);
			// Essentially you can only slash a deposit if it has been taken
			// Deposits are only taken when a role is assigned
			if has_role {
				let deposit_amount: BalanceOf<T> = FellowshipReserves::<T>::get(&who).ok_or(Error::<T>::FellowshipReserveDisapeared)?;
				if slash_deposit {
					let _imbalance = <T as Config>::MultiCurrency::repatriate_reserved(
						CurrencyId::Native,
						&who,
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

    #[derive(Encode, Decode, PartialEq, Eq, Copy, Clone, Debug, MaxEncodedLen, TypeInfo)]
	pub enum Role {
		Vetter, 
		Freelancer,
		BusinessDev,
		Approver,
	}

 }



