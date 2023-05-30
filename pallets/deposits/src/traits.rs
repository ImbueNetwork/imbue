use frame_support::pallet_prelude::*;
use common_types::CurrencyId;
use sp_runtime::traits::{Zero, AtLeast32BitUnsigned};
use codec::{FullCodec, FullEncode};
/// A deposit calculator generic over some type that defines what the storage deposit
/// should be.
pub trait DepositCalculator<Balance> {
    type StorageItem;
    fn calculate_deposit(u: Self::StorageItem, currency: CurrencyId) -> Balance;
}

/// The handler for taking and reinstating deposits.
/// For use in the pallets that need storage deposits.
pub trait DepositHandler<Balance, AccountId> {
    type DepositId: AtLeast32BitUnsigned + Member + TypeInfo + Default + MaxEncodedLen + FullCodec + FullEncode + Copy;
    type StorageItem;

    fn take_deposit(
        who: AccountId,
        storage_item: Self::StorageItem,
        deposit_id: Self::DepositId,
        currency_id: CurrencyId,
    ) -> Result<T::DepositId, DispatchError>;
    fn return_deposit(deposit_id: Self::DepositId) -> DispatchResult;
    fn slash_reserve_deposit(deposit_id: Self::DepositId) -> DispatchResult;
}

#[cfg(feature = "std")]
struct MockDepositHandler<T>(T);
#[cfg(feature = "std")]
impl<T: crate::Config> DepositHandler<crate::BalanceOf<T>, crate::AccountIdOf<T>> for MockDepositHandler<T> {
    type DepositId = T::DepositId;
    type StorageItem = T::StorageItem;
    fn take_deposit(
        _who: crate::AccountIdOf<T>,
        _storage_item: Self::StorageItem,
        _deposit_id: Self::DepositId,
        _currency_id: CurrencyId,
    ) -> Result<T::DepositId, DispatchError> {
        todo!()
    }
    fn return_deposit(_deposit_id: Self::DepositId) -> DispatchResult {
        todo!()
    }
    fn slash_reserve_deposit(_deposit_id: Self::DepositId) -> DispatchResult {
        todo!()
    }
}
