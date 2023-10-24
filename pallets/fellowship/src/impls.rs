use crate::traits::EnsureRole;
use crate::*;
use common_traits::MaybeConvert;
use frame_support::{ensure, traits::Get};
use orml_traits::MultiReservableCurrency;
use sp_runtime::{
    traits::{BadOrigin, Convert},
    DispatchError, Percent,
};
use sp_std::vec::Vec;
use sp_core::H256;

/// Ensure that a account is of a given role.
/// Used in other pallets like an ensure origin.
pub struct EnsureFellowshipRole<T>(T);
impl<T: Config> EnsureRole<AccountIdOf<T>, Role> for EnsureFellowshipRole<T> {
    type Success = ();

    fn ensure_role(
        acc: &AccountIdOf<T>,
        role: Role,
        rank: Option<Rank>,
    ) -> Result<Self::Success, DispatchError> {
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
    ) -> Result<Self::Success, DispatchError> {
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
        }
    }
}

/// Select a jury randomly, if there is not enough member is Roles then a truncated list will be provided.
impl<T: Config> crate::traits::SelectJury<AccountIdOf<T>> for Pallet<T> {
    fn select_jury(mut jury_size: usize) ->  Vec<AccountIdOf<T>> {
        let mut out: Vec<AccountIdOf<T>> = Vec::new();
        let mut rng = rand::thread_rng();
        let mut length = Roles::<T>::iter_keys().count();
        
        if jury_size > length {
            jury_size = length;
        }
        // SAFETY: panics is jury_size > length.
        let sample = rand::seq::index::sample(&mut rng, length, jury_size);

        let mut roles_keys_iterator = Roles::<T>::iter_keys();
        for index in sample.into_vec().iter() {
            if let Some(key) = roles_keys_iterator.nth(*index) {
                out.push(key)
            }
        }

        out
    }
}

impl<T: Config> Pallet<T> {
    /// Try take the membership deposit from who
    /// If the deposit was taken, this will return true, else false.
    pub(crate) fn try_take_deposit(who: &AccountIdOf<T>) -> bool {
        let membership_deposit = <T as Config>::MembershipDeposit::get();
        if <T as Config>::MultiCurrency::reserve(
            T::DepositCurrencyId::get(),
            who,
            membership_deposit,
        )
        .is_ok()
        {
            FellowshipReserves::<T>::insert(who, membership_deposit);
            return true;
        }
        false
    }
}
