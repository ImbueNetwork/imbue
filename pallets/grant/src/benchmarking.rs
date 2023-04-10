//! Benchmarking setup for pallet-grant

use super::*;

#[allow(unused)]
use crate::Pallet as Grant;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {

	impl_benchmark_test_suite!(Grant, crate::mock::new_test_ext(), crate::mock::Test);
}
