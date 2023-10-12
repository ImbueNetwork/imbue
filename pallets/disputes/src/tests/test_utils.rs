pub(super) use crate::traits::*;
pub(super) use crate::{mock::*, pallet::*};
pub(super) use frame_support::traits::Len;
pub(super) use frame_support::{assert_noop, assert_ok, traits::Hooks};
pub(super) use sp_arithmetic::traits::One;
pub(super) use sp_runtime::traits::BlockNumberProvider;
pub(super) use sp_runtime::{BoundedVec, Saturating};

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
