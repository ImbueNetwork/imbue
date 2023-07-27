use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn ensure_role_in_works() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn ensure_role_in_works() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn freelancer_to_vetter_works() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn role_to_percent_doesnt_panic() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn force_add_fellowship_only_force_permitted() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn force_add_fellowship_ok_event_assert() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn leave_fellowship_not_fellow() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn leave_fellowship_assert_event() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_to_fellowship_takes_deposit_if_avaliable() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_to_fellowship_adds_to_pending_fellows_deposit_if_avaliable() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_to_fellowship_adds_vetter_if_exists() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_to_fellowship_edits_role_if_exists_already() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn revoke_fellowship_not_a_fellow() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn revoke_fellowship_unreserves_if_deposit_taken_no_slash() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn revoke_fellowship_slashes_if_deposit_taken_no_slash() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_candidate_to_shortlist_not_a_vetter() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_candidate_to_shortlist_already_fellow() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_candidate_to_shortlist_candidate_lacks_deposit() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_candidate_to_shortlist_candidate_already_on_shortlist() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_candidate_to_shortlist_too_many_candidates() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn add_candidate_to_shortlist_works_assert_event() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn remove_candidate_from_shortlist_not_a_vetter() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn remove_candidate_from_shortlist_works_assert_event() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn pay_deposit_and_remove_pending_status_not_pending() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn pay_deposit_and_remove_pending_status_not_enough_funds_to_reserve() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn pay_deposit_and_remove_pending_status_works_assert_event() {
    new_test_ext().execute_with(|| {});
}

