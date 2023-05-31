use crate::traits::*
use crate::BalanceOf;
use common_runtime::{storage_deposits::StorageDepositItems, currency::DOLLARS};

pub struct ImbueDepositCalculator<T>(T);
impl <T: crate::Config> DepositCalculator<BalanceOf<T>> for ImbueDepositCalculator {
    type StorageItem = StorageDepositItems;
    fn calculate_deposit(u: Self::StorageItem, currency: CurrencyId) -> BalanceOf<T> {
        match StorageItem {
            StorageDepositItems::Project => DOLLARS * 500,
            StorageDepositItems::CrowdFund => DOLLARS * 550,
            StorageDepositItems::Grant => DOLLARS * 400,
            StorageDepositItems::Brief => DOLLARS * 500,
        }
    }
}
    