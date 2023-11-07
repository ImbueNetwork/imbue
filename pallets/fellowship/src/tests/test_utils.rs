use super::*;


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

pub(crate) fn revoke_fellowship(
    who: &AccountIdOf<Test>,
    slash_deposit: bool,
) -> Result<(), DispatchError> {
    <Fellowship as FellowshipHandle<AccountIdOf<Test>>>::revoke_fellowship(who, slash_deposit)
}

pub fn run_to_block<T: Config>(n: BlockNumber) {
    while System::block_number() < n {
        Tokens::on_finalize(System::block_number());
        Fellowship::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Fellowship::on_initialize(System::block_number());
        Tokens::on_initialize(System::block_number());
    }
}
