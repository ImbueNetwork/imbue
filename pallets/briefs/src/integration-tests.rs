// TODO: 
use crate::mock::*;
use crate::*;
use crate::test::*
use common_types::CurrencyId;
use frame_support::pallet_prelude::Hooks;
use frame_support::{assert_noop, assert_ok, once_cell::sync::Lazy};
use sp_core::H256;
use sp_runtime::DispatchError::BadOrigin;
use sp_std::collections::btree_map::BTreeMap;

// all the integration tests for a brief to proposal conversion