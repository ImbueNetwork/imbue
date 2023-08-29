use crate::Rank;
use codec::{FullCodec, FullEncode};
use frame_support::{pallet_prelude::*, weights::Weight};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

/// The democracy handle is used to inititate the regular referenda for new applicants into the fellowship.
pub trait DemocracyHandle<AccountId> {
    fn initiate_shortlist_vote() -> ();
    fn cancel_shortlist_vote() -> ();
}

/// Used by external pallets that decide when to add and remove members from the fellowship.
pub trait FellowshipHandle<AccountId> {
    type Role: Member + TypeInfo + MaxEncodedLen + FullCodec + FullEncode + Copy;
    type Rank: Member + TypeInfo + MaxEncodedLen + FullCodec + FullEncode + Copy;

    fn add_to_fellowship(
        who: &AccountId,
        role: Self::Role,
        rank: Self::Rank,
        vetter: Option<&AccountId>,
        take_membership_deposit: bool,
    );
    fn revoke_fellowship(who: &AccountId, slash_deposit: bool) -> Result<(), DispatchError>;
}

pub trait EnsureRole<AccountId, Role> {
    type Success;
    fn ensure_role(
        acc: &AccountId,
        role: Role,
        maybe_rank: Option<Rank>,
    ) -> Result<Self::Success, DispatchError>;
    fn ensure_role_in(
        acc: &AccountId,
        roles: Vec<Role>,
        maybe_rank: Option<Vec<Rank>>,
    ) -> Result<Self::Success, DispatchError>;
}

pub trait WeightInfoT {
    fn add_to_fellowship() -> Weight;
    fn force_add_fellowship() -> Weight;
    fn leave_fellowship() -> Weight;
    fn force_remove_and_slash_fellowship() -> Weight;
    fn add_candidate_to_shortlist() -> Weight;
    fn remove_candidate_from_shortlist() -> Weight;
    fn pay_deposit_to_remove_pending_status() -> Weight;
}
