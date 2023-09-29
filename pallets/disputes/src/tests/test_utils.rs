pub(crate) use crate::traits::*;
pub(crate) use crate::{mock, mock::*, pallet, pallet::*};
pub(crate) use frame_support::traits::Len;
pub(crate) use frame_support::{assert_noop, assert_ok, traits::Hooks};
pub(crate) use sp_arithmetic::traits::One;
pub(crate) use sp_runtime::traits::BlockNumberProvider;
pub(crate) use sp_runtime::{BoundedBTreeMap, BoundedVec, Saturating};

pub fn run_to_block<T: Config>(n: T::BlockNumber)
where
    T::BlockNumber: Into<u64>,
{
    loop {
        let mut block: T::BlockNumber = frame_system::Pallet::<T>::block_number();
        if block >= n {
            break;
        }
        block = block.saturating_add(<T::BlockNumber as One>::one());
        frame_system::Pallet::<T>::set_block_number(block);
        frame_system::Pallet::<T>::on_initialize(block);
        PalletDisputes::on_initialize(block.into());
    }
}

pub fn get_jury<T: Config>(
    accounts: Vec<AccountIdOf<T>>,
) -> BoundedVec<AccountIdOf<T>, <T as Config>::MaxJurySize> {
    accounts.try_into().expect("too many jury members")
}

pub fn get_specifics<T: Config>(
    specifics: Vec<T::SpecificId>,
) -> BoundedVec<T::SpecificId, T::MaxSpecifics> {
    specifics.try_into().expect("too many specific ids.")
}

#[cfg(feature = "runtime-benchmarks")]
pub fn create_funded_user<T: Config>(
    seed: &'static str,
    n: u32,
    balance_factor: u128,
) -> T::AccountId {
    let user = account(seed, n, 0);
    assert_ok!(<T::MultiCurrency as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::deposit(
        CurrencyId::Native, &user, balance_factor.saturated_into()
    ));
    user
}
