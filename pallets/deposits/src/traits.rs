

use codec::FullCodec;
use frame_support::pallet_prelude::*;
use frame_support::dispatch::fmt::Debug;

/// A deposit calculator generic over some type that defines what the storage deposit
/// should be.
pub trait DepositCalculator<Balance> {
    type CurrencyId: Clone + Copy + PartialOrd + Ord + PartialEq + Eq + Debug + Encode + Decode + TypeInfo + MaxEncodedLen;
    type StorageItem;

    fn calculate_deposit(u: Self::StorageItem, currency: Self::CurrencyId) -> Balance;
}

// TODO: do i need to bind these associated types or is the bound in the config enough..
/// The handler for taking and reinstating deposits.
pub trait DepositHandler<Balance, AccountId> {
    type DepositId;
    type StorageItem;
    type CurrencyId: Clone + Copy + PartialOrd + Ord + PartialEq + Eq + Debug + Encode + Decode + TypeInfo + MaxEncodedLen;
    fn take_deposit(who: AccountId, deposit_id: Self::DepositId, item: Self::StorageItem, currency_id: Self::CurrencyId) -> DispatchResult;
    fn reinstate_deposit(deposit_id: Self::DepositId) -> DispatchResult;
}