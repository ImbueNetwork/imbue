//! Benchmarking setup for pallet-grants

use super::*;

#[allow(unused)]
use crate::Pallet as Grant;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
