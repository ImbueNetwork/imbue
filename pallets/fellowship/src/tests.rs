use crate::impls::*;
use crate::traits::*;
use crate::Pallet as FellowshipI;
use crate::*;
use crate::{mock::*, Error, Event, FellowToVetter, Role, Roles};
use common_traits::MaybeConvert;
use common_types::CurrencyId;
use frame_support::{assert_err, assert_noop, assert_ok, once_cell::sync::Lazy};
use frame_system::Pallet as System;
use sp_std::{vec, vec::Vec};
use sp_runtime::traits::{BadOrigin};
use orml_traits::MultiReservableCurrency;

// Saves a bit of typing.
type Fellowship = FellowshipI<Test>;
pub(crate) static dep_currency: Lazy<CurrencyId> = Lazy::new(|| <Test as Config>::DepositCurrencyId::get());

#[test]
fn ensure_role_in_works() {
    new_test_ext().execute_with(|| {
        Roles::<Test>::insert(*ALICE, (Role::Vetter, 10));
        Roles::<Test>::insert(*BOB, (Role::Freelancer, 10));

        assert_ok!(EnsureFellowshipRole::<Test>::ensure_role_in(
            &ALICE,
            vec![Role::Vetter, Role::Freelancer],
            None
        ));
        assert_ok!(EnsureFellowshipRole::<Test>::ensure_role_in(
            &BOB,
            vec![Role::Vetter, Role::Freelancer],
            None
        ));
        assert!(
            EnsureFellowshipRole::<Test>::ensure_role_in(&BOB, vec![Role::Approver], None).is_err(),
            "BOB is not of this Role."
        );
        assert!(
            EnsureFellowshipRole::<Test>::ensure_role_in(&ALICE, vec![Role::Freelancer], None)
                .is_err(),
            "ALICE is not of this Role."
        );
    });
}

#[test]
fn ensure_role_in_works_with_rank() {
    new_test_ext().execute_with(|| {
        Roles::<Test>::insert(*ALICE, (Role::Vetter, 10));
        assert_ok!(EnsureFellowshipRole::<Test>::ensure_role_in(
            &ALICE,
            vec![Role::Vetter],
            Some(vec![10, 9])
        ));

        assert_noop!(
            EnsureFellowshipRole::<Test>::ensure_role_in(&ALICE, vec![Role::Vetter], Some(vec![9])),
            BadOrigin
        );
    });
}

#[test]
fn ensure_role_works() {
    new_test_ext().execute_with(|| {
        Roles::<Test>::insert(*ALICE, (Role::Vetter, 0));
        assert_ok!(EnsureFellowshipRole::<Test>::ensure_role(
            &ALICE,
            Role::Vetter,
            None
        ));
        assert!(EnsureFellowshipRole::<Test>::ensure_role(&ALICE, Role::Freelancer, None).is_err());
    });
}

#[test]
fn ensure_role_works_with_rank() {
    new_test_ext().execute_with(|| {
        Roles::<Test>::insert(*ALICE, (Role::Vetter, 10));
        assert_ok!(EnsureFellowshipRole::<Test>::ensure_role(
            &ALICE,
            Role::Vetter,
            Some(10)
        ));
        
        assert_noop!(EnsureFellowshipRole::<Test>::ensure_role(
            &ALICE,
            Role::Vetter,
            Some(9)
        ), BadOrigin);
    });
}

#[test]
fn freelancer_to_vetter_works() {
    new_test_ext().execute_with(|| {
        FellowToVetter::<Test>::insert(*ALICE, *BOB);
        let v = <Fellowship as MaybeConvert<&AccountIdOf<Test>, VetterIdOf<Test>>>::maybe_convert(&ALICE).expect("we just inserted so should be there.");
        assert_eq!(v, *BOB);
        assert!(<Fellowship as MaybeConvert<&AccountIdOf<Test>, VetterIdOf<Test>>>::maybe_convert(&BOB).is_none());
    });
}

#[test]
fn force_add_fellowship_only_force_permitted() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Fellowship::force_add_fellowship(
                RuntimeOrigin::signed(*ALICE),
                *BOB,
                Role::Freelancer,
                10
            ),
            BadOrigin
        );
    });
}

#[test]
fn force_add_fellowship_ok_event_assert() {
    new_test_ext().execute_with(|| {
        assert_ok!(Fellowship::force_add_fellowship(
            RuntimeOrigin::root(),
            *BOB,
            Role::Freelancer,
            10
        ));
        System::<Test>::assert_last_event(
            Event::<Test>::FellowshipAdded {
                who: *BOB,
                role: Role::Freelancer,
            }
            .into(),
        );
    });
}

#[test]
fn leave_fellowship_not_fellow() {
    new_test_ext().execute_with(|| {
        assert_noop!(Fellowship::leave_fellowship(RuntimeOrigin::signed(*ALICE)), Error::<Test>::NotAFellow);
    });  
}

#[test]
fn force_add_fellowship_then_leave_fellowship_maintains_reserve_amount() {
    new_test_ext().execute_with(|| {
        let alice_reserved_before = <Test as Config>::MultiCurrency::reserved_balance(*dep_currency, &ALICE);
        let _ = Fellowship::force_add_fellowship(RuntimeOrigin::root(), *ALICE, Role::Freelancer, 10).expect("qed");
        let alice_reserved_after = <Test as Config>::MultiCurrency::reserved_balance(*dep_currency, &ALICE);
        assert_eq!(alice_reserved_before, alice_reserved_after);
    });
}

#[test]
fn leave_fellowship_assert_event() {
    new_test_ext().execute_with(|| {
        let _ = Fellowship::force_add_fellowship(RuntimeOrigin::root(), *ALICE, Role::Freelancer, 10).expect("qed");



    });
}

#[test]
fn add_to_fellowship_takes_deposit_if_avaliable() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_to_fellowship_adds_to_pending_fellows_deposit_if_avaliable() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_to_fellowship_adds_vetter_if_exists() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_to_fellowship_edits_role_if_exists_already() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn revoke_fellowship_not_a_fellow() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn revoke_fellowship_unreserves_if_deposit_taken_no_slash() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn revoke_fellowship_slashes_if_deposit_taken_no_slash() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_candidate_to_shortlist_not_a_vetter() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_candidate_to_shortlist_already_fellow() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_candidate_to_shortlist_candidate_lacks_deposit() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_candidate_to_shortlist_candidate_already_on_shortlist() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_candidate_to_shortlist_too_many_candidates() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn add_candidate_to_shortlist_works_assert_event() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn remove_candidate_from_shortlist_not_a_vetter() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn remove_candidate_from_shortlist_works_assert_event() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn pay_deposit_and_remove_pending_status_not_pending() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn pay_deposit_and_remove_pending_status_not_enough_funds_to_reserve() {
    new_test_ext().execute_with(|| assert!(false));
}

#[test]
fn pay_deposit_and_remove_pending_status_works_assert_event() {
    new_test_ext().execute_with(|| assert!(false));
}
