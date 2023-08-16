use sp_runtime::{DispatchError, traits::AtLeast32BitUnsigned};

pub trait DisputeRaiser<AccountId> {
    type DisputeKey: AtLeast32BitUnsigned;

    // Strip this to be the minumim the dispute pallet needs to know.
    // where is the money,
    // Who is the jury,
    // Bind the string to a constant amount (500)
    fn raise_dispute(
        raised_by: AccountId,
        fund_account: AccountId,
        reason: Vec<u8>,
        project_id: u32,
        jury: Vec<AccountId>,
    ) -> Result<(), DispatchError>;
}

pub trait JurySelector{
     fn select_jury()-> Result<(), DispatchError>;
}