pub(super) use crate::traits::*;
pub(super) use crate::{mock::*, pallet::*};
pub(super) use frame_support::traits::Len;
pub(super) use frame_support::{assert_noop, assert_ok, traits::Hooks};
pub(super) use sp_runtime::traits::BlockNumberProvider;
pub(super) use sp_runtime::BoundedVec;

pub fn run_to_block<T: Config>(n: BlockNumber)
{
    while System::block_number() < n {
        Tokens::on_finalize(System::block_number());
        PalletDisputes::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        PalletDisputes::on_initialize(System::block_number());
        Tokens::on_initialize(System::block_number());
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
