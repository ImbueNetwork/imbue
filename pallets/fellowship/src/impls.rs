use crate::traits::EnsureRole;
use sp_runtime::traits::BadOrigin;
use crate::{VetterIdOf<T>, AccountIdOf<T>}; 

/// Ensure that a account is of a given role.
/// Used in other pallets like an ensure origin.
pub struct EnsureFellowshipRole<T>(T);
impl<T: Config> EnsureRole<AccountIdOf<T>, Role> for EnsureFellowshipRole<T> {
	type Success = ();
	
	fn ensure_role(acc: &AccountIdOf<T>, role: Role) -> Result<Self::Success, BadOrigin> {
		let actual = Roles::<T>::get(acc).ok_or(BadOrigin)?;
		if role == actual {
			Ok(())
		} else {
			Err(BadOrigin)
		}
	}
	fn ensure_role_in(acc: &AccountIdOf<T>, roles: Vec<Role>) -> Result<Self::Success, BadOrigin> {
		let role = Roles::<T>::get(acc).ok_or(BadOrigin)?;
		ensure!(roles.contains(&role), BadOrigin);
		Ok(())
	}
}

impl<T: Config> TryConvert<&AccountIdOf<T>, VetterIdOf<T>> for Pallet<T> {
	fn try_convert(fellow: &AccountIdOf<T>) -> Result<VetterIdOf<T>, &AccountIdOf<T>> {
		if let Some(vetter) = FellowToVetter::<T>::get(fellow) {
			Ok(vetter)
		} else {
			Err(fellow)
		}
	}
}

pub struct RoleToPercent;
impl<Role> Convert<Role, Percent> for Pallet<T> {
	fn convert(role: &AccountIdOf<T>) -> Result<VetterIdOf<T>, &AccountIdOf<T>> {
		
	}
}