
use codec::{FullEncode, FullCodec, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, traits::AtLeast32BitUnsigned, BoundedVec};


pub trait DisputeRaiser<AccountId> {
    type DisputeKey: AtLeast32BitUnsigned + FullEncode + FullCodec + MaxEncodedLen + TypeInfo;
    type MaxReasonLength: Get<u32>;
    type MaxJurySize: Get<u32>;

    // Strip this to be the minumim the dispute pallet needs to know.
    // where is the money,
    // Who is the jury,
    // Bind the string to a constant amount (500)
    fn raise_dispute(
        dispute_key: Self::DisputeKey,
        raised_by: AccountId,
        fund_account: AccountId,
        reason: BoundedVec<u8, Self::MaxReasonLength>,
        jury: BoundedVec<AccountId, Self::MaxJurySize>,
    ) -> Result<(), DispatchError>;
}


pub trait DisputeHooks<DisputeKey> {
    // Outcome
    // handle the completed dispute
    fn on_dispute_complete(dispute_key: DisputeKey,) -> Result<(), DispatchError>;
    fn on_dispute_cancel(dispute_key: DisputeKey) -> Result<(), DispatchError>;
}


