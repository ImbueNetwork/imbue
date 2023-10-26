use crate::impls::*;
use crate::traits::*;
use crate::*;
use crate::{mock::*, Error, Event, FellowToVetter, Role, Roles};
use common_traits::MaybeConvert;
use common_types::CurrencyId;
use frame_support::{
    assert_noop, assert_ok, once_cell::sync::Lazy, traits::Hooks, BoundedBTreeMap,
};
use frame_system::Pallet as System;
use orml_tokens::Error as TokensError;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_arithmetic::traits::One;
use sp_core::sr25519::Public;
use sp_runtime::{traits::BadOrigin, DispatchError, Saturating};
use sp_std::vec;

pub(crate) static DEP_CURRENCY: Lazy<CurrencyId> =
    Lazy::new(<Test as Config>::DepositCurrencyId::get);

mod pallet_tests;
mod ensure_role;
mod fellowship_permissions;
mod test_utils;
pub(crate) use test_utils::*;