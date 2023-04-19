use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn ensure_milestone_percent_equal_100() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}

#[test]
fn create_grant_already_exists() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}


#[test]
fn edit_grant_only_submitter_can_edit() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}


#[test]
fn edit_grant_not_found() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}