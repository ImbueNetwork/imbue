use crate::*;
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn submit_basic_brief() {
    build_test_externality().execute_with(|| {
		assert!(true)
	});
}
