use frame_support::pallet_prelude::*;
use common_types::CurrencyId;
use sp_runtime::traits::{Zero, AtLeast32BitUnsigned};
use codec::{FullCodec, FullEncode};

/// A deposit calculator generic over some type that defines what the storage deposit
/// should be.
pub trait DepositCalculator<Balance> {
    type StorageItem;
    fn calculate_deposit(u: Self::StorageItem, currency: CurrencyId) -> Result<Balance, ()>;
}

/// The handler for taking and reinstating deposits.
/// For use in the pallets that need storage deposits.
pub trait DepositHandler<Balance, AccountId> {
    type DepositId: AtLeast32BitUnsigned + Member + TypeInfo + Default + MaxEncodedLen + FullCodec + FullEncode + Copy;
    type StorageItem;

    fn take_deposit(
        who: AccountId,
        storage_item: Self::StorageItem,
        currency_id: CurrencyId,
    ) -> Result<Self::DepositId, DispatchError>;
    fn return_deposit(deposit_id: Self::DepositId) -> DispatchResult;
    fn slash_reserve_deposit(deposit_id: Self::DepositId) -> DispatchResult;
}

