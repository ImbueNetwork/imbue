use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn ensure_milestone_percent_equal_100() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}

#[test]
fn create_grant_too_many_in_exp_block() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}

#[test]
fn vote_on_grant_not_found() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}

#[test]
fn only_approvers_can_vote() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}

#[test]
fn expired_grants_are_actually_removed() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}

#[test]
fn expired_grant_removed_in_hook_fails_gracefully_if_does_not_exist() {
	new_test_ext().execute_with(|| {
		assert!(false);
	});
}