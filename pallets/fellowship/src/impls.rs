use crate::traits::EnsureRole;
use crate::*;
use common_traits::MaybeConvert;
use frame_support::ensure;
use sp_runtime::{
    traits::{BadOrigin, Convert},
    Percent,
};
use sp_std::vec::Vec;

/// Ensure that a account is of a given role.
/// Used in other pallets like an ensure origin.
pub struct EnsureFellowshipRole<T>(T);
impl<T: Config> EnsureRole<AccountIdOf<T>, Role> for EnsureFellowshipRole<T> {
    type Success = ();

    fn ensure_role(
        acc: &AccountIdOf<T>,
        role: Role,
        rank: Option<Rank>,
    ) -> Result<Self::Success, BadOrigin> {
        let (actual_role, actual_rank) = Roles::<T>::get(acc).ok_or(BadOrigin)?;
        ensure!(actual_role == role, BadOrigin);
        if let Some(r) = rank {
            ensure!(r == actual_rank, BadOrigin);
        }
        Ok(())
    }
    fn ensure_role_in(
        acc: &AccountIdOf<T>,
        roles: Vec<Role>,
        ranks: Option<Vec<Rank>>,
    ) -> Result<Self::Success, BadOrigin> {
        let (actual_role, actual_rank) = Roles::<T>::get(acc).ok_or(BadOrigin)?;
        ensure!(roles.contains(&actual_role), BadOrigin);
        if let Some(r) = ranks {
            ensure!(r.contains(&actual_rank), BadOrigin);
        }
        Ok(())
    }
}

impl<T: Config> MaybeConvert<&AccountIdOf<T>, VetterIdOf<T>> for Pallet<T> {
    fn maybe_convert(fellow: &AccountIdOf<T>) -> Option<VetterIdOf<T>> {
        FellowToVetter::<T>::get(fellow)
    }
}

pub struct RoleToPercentFee;
impl Convert<crate::Role, Percent> for RoleToPercentFee {
    fn convert(role: Role) -> Percent {
        match role {
            Role::Vetter => Percent::from_percent(50u8),
            Role::Freelancer => Percent::from_percent(50u8),
            Role::BusinessDev => Percent::from_percent(50u8),
            Role::Approver => Percent::from_percent(50u8),
        }
    }
}
