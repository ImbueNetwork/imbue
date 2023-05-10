//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v1::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

//benchmarks! {
//	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
//}
