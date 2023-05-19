

use codec::FullCodec;
use frame_support::pallet_prelude::*;
use frame_support::dispatch::fmt::Debug;

/// A deposit calculator generic over some type that defines what the storage deposit
/// should be.
pub trait DepositCalculator<Balance> {
    type CurrencyId;
    type StorageItem;

    fn calculate_deposit(u: Self::StorageItem, currency: Self::CurrencyId) -> Balance;
}

/// The handler for taking and reinstating deposits.
/// For use in the pallets that need storage deposits.
pub trait DepositHandler<Balance, AccountId> {
    type DepositId;
    type StorageItem;
    type CurrencyId;
    fn take_deposit(who: AccountId, deposit_id: Self::DepositId, item: Self::StorageItem, currency_id: Self::CurrencyId) -> DispatchResult;
    fn return_deposit(deposit_id: Self::DepositId) -> DispatchResult;
    fn slash_reserve_deposit(deposit_id: Self::DepositId) -> DispatchResult;
}