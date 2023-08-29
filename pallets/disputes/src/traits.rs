
Euse codec::{FullEncode, FullCodec, MaxEncodedLen};
use frame_system::Config;
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, traits::AtLeast32BitUnsigned, BoundedVec};


pub trait DisputeRaiser<AccountId> {
    type DisputeKey: AtLeast32BitUnsigned + FullEncode + FullCodec + MaxEncodedLen + TypeInfo;

    // Strip this to be the minumim the dispute pallet needs to know.
    // where is the money,
    // Who is the jury,
    // Bind the string to a constant amount (500)
    fn raise_dispute(
        dispute_key: Self::DisputeKey,
        raised_by: AccountId,
        fund_account: AccountId,
        reason: BoundedVec<u8, <T as Config>::MaxReasonLength>,
        project_id: u32,
        jury: Vec<AccountId>,
    ) -> Result<(), DispatchError>;
}

pub trait DisputeHooks<DisputeKey> {
    // Outcome
    // handle the completed dispute
    fn on_dispute_complete() -> ();
}

