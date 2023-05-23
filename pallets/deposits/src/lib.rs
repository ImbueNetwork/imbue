#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod traits;

#[frame_support::pallet]
pub mod pallet {
    use crate::traits::{DepositCalculator, DepositHandler};
    use codec::FullCodec;
    use frame_support::dispatch::fmt::Debug;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use orml_traits::{BalanceStatus, MultiCurrency, MultiReservableCurrency};
    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type CurrencyId: Clone
            + Copy
            + PartialOrd
            + Ord
            + PartialEq
            + Eq
            + Debug
            + Encode
            + Decode
            + TypeInfo
            + MaxEncodedLen;
        type MultiCurrency: MultiReservableCurrency<
            AccountIdOf<Self>,
            CurrencyId = Self::CurrencyId,
        >;
        /// The ID used to differentitate storage types.
        type DepositId: FullCodec + Copy + Eq + PartialEq + Debug + MaxEncodedLen + TypeInfo;
        /// The actual types that are being put in storage, abstracted as an enum;
        type StorageItem: FullCodec + Eq + PartialEq + Copy + Debug;
        /// The type responsible for calculating the cost of a storage item based on its type.
        type DepositCalculator: DepositCalculator<
            BalanceOf<Self>,
            CurrencyId = Self::CurrencyId,
            StorageItem = Self::StorageItem,
        >;
        /// The account slashed deposits are sent to.
        type DepositSlashAccount: Get<AccountIdOf<Self>>;
    }

    /// A list of current deposits and the amount taken for the deposit.
    #[pallet::storage]
    pub type CurrentDeposits<T> =
        StorageMap<_, Blake2_128, <T as Config>::DepositId, Deposit<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A deposit has been taken.
        DepositTaken(T::DepositId, BalanceOf<T>),
        /// A deposit has been reinstated.
        DepositReturned(T::DepositId, BalanceOf<T>),
        DepositSlashed(T::DepositId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A deposit has already been taken for this key.
        DepositAlreadyExists,
        /// The deposit doesnt exist.
        DepositDoesntExist,
    }

    impl<T: Config> DepositHandler<BalanceOf<T>, AccountIdOf<T>> for Pallet<T> {
        type CurrencyId = T::CurrencyId;
        type StorageItem = T::StorageItem;
        type DepositId = T::DepositId;

        /// Take a deposit using a given id, if using multiple types take care to ensure the ids do not collide.
        fn take_deposit(
            who: AccountIdOf<T>,
            deposit_id: Self::DepositId,
            item: Self::StorageItem,
            currency_id: T::CurrencyId,
        ) -> DispatchResult {
            ensure!(
                !CurrentDeposits::<T>::contains_key(&deposit_id),
                Error::<T>::DepositAlreadyExists
            );
            let amount = <T as Config>::DepositCalculator::calculate_deposit(item, currency_id);
            <T as Config>::MultiCurrency::reserve(currency_id, &who, amount)?;
            let deposit = Deposit {
                who,
                amount,
                currency_id,
            };
            CurrentDeposits::<T>::insert(deposit_id, deposit);
            Self::deposit_event(Event::<T>::DepositTaken(deposit_id, amount));

            Ok(().into())
        }

        fn return_deposit(deposit_id: T::DepositId) -> DispatchResult {
            let deposit =
                CurrentDeposits::<T>::get(deposit_id).ok_or(Error::<T>::DepositDoesntExist)?;
            <T as Config>::MultiCurrency::unreserve(
                deposit.currency_id,
                &deposit.who,
                deposit.amount,
            );

            CurrentDeposits::<T>::remove(deposit_id);
            Self::deposit_event(Event::<T>::DepositReturned(deposit_id, deposit.amount));
            Ok(().into())
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
            Ok(().into())
        }
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Deposit<T: Config> {
        who: AccountIdOf<T>,
        amount: BalanceOf<T>,
        currency_id: T::CurrencyId,
    }
}
