#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod traits;

#[frame_support::pallet]
pub mod pallet {
    use crate::traits::{DepositCalculator, DepositHandler};
    use codec::{FullCodec, FullEncode};
    use common_types::CurrencyId;
    use frame_support::pallet_prelude::*;
    use orml_traits::{BalanceStatus, MultiCurrency, MultiReservableCurrency};
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, One},
        Saturating,
    };
    use sp_std::fmt::Debug;
    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MultiCurrency: MultiReservableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;
        /// The actual storage types you want to take deposits for, abstracted as an enum.
        type StorageItem: Copy + Eq + PartialEq + Debug;

        type DepositId: AtLeast32BitUnsigned
            + Member
            + TypeInfo
            + Default
            + MaxEncodedLen
            + FullCodec
            + FullEncode
            + Copy;

        /// The type responsible for calculating the cost of a storage item based on some DepositId.
        type DepositCalculator: DepositCalculator<BalanceOf<Self>, StorageItem = Self::StorageItem>;
        /// The account slashed deposits are sent to.
        type DepositSlashAccount: Get<AccountIdOf<Self>>;
    }

    /// A list of current deposits and the amount taken for the deposit.
    #[pallet::storage]
    pub type CurrentDeposits<T> =
        StorageMap<_, Blake2_128, <T as Config>::DepositId, Deposit<T>, OptionQuery>;

    /// A counter for generating DepositIds;
    #[pallet::storage]
    pub type TicketId<T> = StorageValue<_, <T as Config>::DepositId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A deposit has been taken.
        DepositTaken(T::DepositId, BalanceOf<T>),
        /// A deposit has been reinstated.
        DepositReturned(T::DepositId, BalanceOf<T>),
        /// A deposit has been slashed and sent to the slash account.
        DepositSlashed(T::DepositId, BalanceOf<T>),
        /// A deposit has been ignored due to u32::MAX being passed.
        DepositIgnored,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The deposit doesnt exist.
        DepositDoesntExist,
        /// The currency type is not supported.
        UnsupportedCurrencyType,
        /// The storage type is not supported.
        UnsupportedStorageType,
        /// You need more funds to cover the storage deposit.
        NotEnoughFundsForStorageDeposit,
    }

    impl<T: Config> DepositHandler<BalanceOf<T>, AccountIdOf<T>> for Pallet<T> {
        type DepositId = T::DepositId;
        type StorageItem = T::StorageItem;

        /// Take a deposit from an account, the cost of a deposit is specified by the StorageItem.
        /// This will return a DepositId which is like the ticket you get using a cloakroom.
        /// The ticket is then used to return the deposit.
        fn take_deposit(
            who: AccountIdOf<T>,
            storage_item: T::StorageItem,
            currency_id: CurrencyId,
        ) -> Result<T::DepositId, DispatchError> {
            let amount =
                <T as Config>::DepositCalculator::calculate_deposit(storage_item, currency_id)?;
            let deposit_id = Self::get_new_deposit_id();
            <T as Config>::MultiCurrency::reserve(currency_id, &who, amount)
                .map_err(|_| Error::<T>::NotEnoughFundsForStorageDeposit)?;
            let deposit = Deposit {
                who,
                amount,
                currency_id,
            };
            CurrentDeposits::<T>::insert(deposit_id, deposit);
            Self::deposit_event(Event::<T>::DepositTaken(deposit_id, amount));
            Ok(deposit_id)
        }

        /// Given a deposit id (the ticket generated when creating a deposit) return the deposit.
        /// If a deposit_id of u32::MAX is passed, the deposit_id will be ignored and nothing will be returned.
        /// This should allow for easier migration of types.
        fn return_deposit(deposit_id: T::DepositId) -> DispatchResult {
            if deposit_id == u32::MAX.into() {
                Self::deposit_event(Event::<T>::DepositIgnored);
                return Ok(());
            }
            let deposit =
                CurrentDeposits::<T>::get(deposit_id).ok_or(Error::<T>::DepositDoesntExist)?;
            <T as Config>::MultiCurrency::unreserve(
                deposit.currency_id,
                &deposit.who,
                deposit.amount,
            );

            CurrentDeposits::<T>::remove(deposit_id);
            Self::deposit_event(Event::<T>::DepositReturned(deposit_id, deposit.amount));
            Ok(())
        }

        /// Doesnt do a slash in the normal sense, this simply takes the deposit and sends it to the DepositSlashAccount.
        fn slash_reserve_deposit(deposit_id: T::DepositId) -> DispatchResult {
            let deposit =
                CurrentDeposits::<T>::get(deposit_id).ok_or(Error::<T>::DepositDoesntExist)?;
            let beneficiary = &<T as Config>::DepositSlashAccount::get();
            // TODO: if the reserve amount is returned then take from free balance?
            let _imbalance = <T as Config>::MultiCurrency::repatriate_reserved(
                deposit.currency_id,
                &deposit.who,
                beneficiary,
                deposit.amount,
                BalanceStatus::Free,
            )?;

            CurrentDeposits::<T>::remove(deposit_id);
            Self::deposit_event(Event::<T>::DepositSlashed(deposit_id, deposit.amount));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Generate a DepositId, used as a ticket. Infallible.
        pub(crate) fn get_new_deposit_id() -> T::DepositId {
            let ticket_id = TicketId::<T>::get();
            TicketId::<T>::put(ticket_id.saturating_add(One::one()));
            ticket_id
        }
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Deposit<T: Config> {
        who: AccountIdOf<T>,
        amount: BalanceOf<T>,
        currency_id: CurrencyId,
    }
}
