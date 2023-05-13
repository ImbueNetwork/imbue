use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn milestones_must_sum_to_100_on_creation() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn below_minimum_required_funds() {
	new_test_ext().execute_with(|| {

	});
}

