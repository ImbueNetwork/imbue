use codec::{FullCodec, FullEncode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::AtLeast32BitUnsigned, BoundedVec, DispatchError};
use sp_std::vec::Vec;

pub trait DisputeRaiser<AccountId> {
    type DisputeKey: AtLeast32BitUnsigned + FullEncode + FullCodec + MaxEncodedLen + TypeInfo;
    type SpecificId: AtLeast32BitUnsigned + FullEncode + FullCodec + MaxEncodedLen + TypeInfo;
    type MaxJurySize: Get<u32>;
    type MaxSpecifics: Get<u32>;

    fn raise_dispute(
        dispute_key: Self::DisputeKey,
        raised_by: AccountId,
        jury: BoundedVec<AccountId, Self::MaxJurySize>,
        specific_ids: BoundedVec<Self::SpecificId, Self::MaxSpecifics>,
    ) -> Result<(), DispatchError>;
}

pub trait DisputeHooks<DisputeKey, SpecificId, AccountId> {
    /// On the completion of a dispute, this hooks is called.
    /// Returning only the key that has been handled and the result of the dispute.
    fn on_dispute_complete(
        raised_by: AccountId,
        dispute_key: DisputeKey,
        specifics: Vec<SpecificId>,
        dispute_result: crate::pallet::DisputeResult,
    ) -> Weight;
}
