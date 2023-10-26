use super::*;

pub(crate) static DEP_CURRENCY: Lazy<CurrencyId> =
    Lazy::new(<Test as Config>::DepositCurrencyId::get);

// Saves a bit of typing.
pub(crate) fn add_to_fellowship_take_deposit(
    who: &AccountIdOf<Test>,
    role: Role,
    rank: Rank,
    vetter: Option<&VetterIdOf<Test>>,
) -> Result<(), DispatchError> {
    <Fellowship as FellowshipHandle<AccountIdOf<Test>>>::add_to_fellowship(
        who, role, rank, vetter, true,
    );
    Ok(())
}

pub(crate) fn revoke_fellowship(who: &AccountIdOf<Test>, slash_deposit: bool) -> Result<(), DispatchError> {
    <Fellowship as FellowshipHandle<AccountIdOf<Test>>>::revoke_fellowship(who, slash_deposit)
}

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
        Fellowship::on_initialize(block.into());
    }
}
