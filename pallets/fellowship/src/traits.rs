use sp_runtime::traits::BadOrigin;
use frame_support::pallet_prelude::*;
use codec::{FullCodec, FullEncode};

/// The democracy handle is used to inititate the regular referenda for new applicants into the fellowship.
pub trait DemocracyHandle<AccountId> {
    fn initiate_shortlist_vote() -> ();
    fn cancel_shortlist_vote() -> ();
}

/// Used by external pallets that decide when to add and remove members from the fellowship.
/// Makes a ying/yang with the democracy handle :)
pub trait FellowshipHandle<AccountId> {
    type Role: Member
    + TypeInfo
    + MaxEncodedLen
    + FullCodec
    + FullEncode
    + Copy;

    fn add_to_fellowship(who: &AccountId, role: Self::Role) -> Result<(), DispatchError>;
    fn revoke_fellowship(who: &AccountId, slash_deposit: bool) -> Result<(), DispatchError>;
    fn bulk_add_to_fellowship(shortlist: Vec<(AccountId, Self::Role)>) -> Result<(), DispatchError>;
}

pub trait EnsureRole<AccountId, Role> {
    type Success;
    fn ensure_role(acc: &AccountId, role: Role) -> Result<Self::Success, BadOrigin>;
    fn ensure_role_in(acc: &AccountId, roles: Vec<Role>) -> Result<Self::Success, BadOrigin>;
}
