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
            EnsureFellowshipRole::<Test>::ensure_role_in(&BOB, vec![Role::Vetter], None).is_err(),
            "BOB is not of this Role."
        );
        assert!(
            EnsureFellowshipRole::<Test>::ensure_role_in(&ALICE, vec![Role::Freelancer], None)
                .is_err(),
            "ALICE is not of this Role."
        );
    });
}

use super::*;

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

        assert_noop!(
            EnsureFellowshipRole::<Test>::ensure_role(&ALICE, Role::Vetter, Some(9)),
            BadOrigin
        );
    });
}
