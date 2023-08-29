use sp_runtime::{DispatchError, traits::AtLeast32BitUnsigned};

pub trait DisputeRaiser<AccountId> {
    type DisputeKey: AtLeast32BitUnsigned + FullEncode + FullCodec + MaxEncodedLen + TypeInfo;

    // Strip this to be the minumim the dispute pallet needs to know.
    // where is the money,
    // Who is the jury,
    // Bind the string to a constant amount (500)
    fn raise_dispute(
        dispute_key: Self::DisputeKey,
        raised_by: AccountIdOf<T>,
        fund_account: AccountIdOf<T>,
        reason: Vec<u8>,
        project_id: u32,
        jury: Vec<AccountIdOf<T>>,
    ) -> Result<(), DispatchError>;
}

pub trait DisputeHooks<DisputeKey> {
    // Outcome
    // handle the completed dispute
    fn on_dispute_complete() -> ();
    fn on_dispute_cancel() -> ();
}

enum Outcome {
    Refund, 
    ContinueNormally,
    Slash
}

