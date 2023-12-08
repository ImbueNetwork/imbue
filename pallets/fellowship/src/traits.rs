use crate::{Rank, Role};
use codec::{FullCodec, FullEncode};
use frame_support::{pallet_prelude::*, weights::Weight};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

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

pub trait EnsureRole<AccountId> {
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

/// Select a pseudo-random jury of a specified amount.
pub trait SelectJury<AccountId> {
    type JurySize: Get<u32>;
    fn select_jury() -> BoundedVec<AccountId, Self::JurySize>;
}

/// Custom definition for permissions for each role.
pub trait FellowshipPermissions<Role, Permission> {
    fn has_permission(role: Role, permission: Permission) -> bool;
    fn get_permissions(role: Role) -> Vec<Permission>;
}

