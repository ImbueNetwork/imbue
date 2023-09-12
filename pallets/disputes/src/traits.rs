use codec::{FullCodec, FullEncode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::AtLeast32BitUnsigned, BoundedVec, DispatchError};

pub trait DisputeRaiser<AccountId> {
    type DisputeKey: AtLeast32BitUnsigned + FullEncode + FullCodec + MaxEncodedLen + TypeInfo;
    type SpecificId: AtLeast32BitUnsigned + FullEncode + FullCodec + MaxEncodedLen + TypeInfo;
    type MaxReasonLength: Get<u32>;
    type MaxJurySize: Get<u32>;
    type MaxSpecifics: Get<u32>;

    fn raise_dispute(
        dispute_key: Self::DisputeKey,
        raised_by: AccountId,
        jury: BoundedVec<AccountId, Self::MaxJurySize>,
        specific_ids: BoundedVec<Self::SpecificId, Self::MaxSpecifics>, 
    ) -> Result<(), DispatchError>;
}

pub trait DisputeHooks<DisputeKey> {
    // Outcome
    // handle the completed dispute
    fn on_dispute_complete(dispute_key: DisputeKey) -> Result<(), DispatchError>;
    fn on_dispute_cancel(dispute_key: DisputeKey) -> Result<(), DispatchError>;
}
