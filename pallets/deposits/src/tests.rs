use crate::mock::*;
use crate::traits::{DepositCalculator, DepositHandler};
use crate::*;
use frame_support::assert_ok;

#[test]
fn get_new_deposit_id_works() {
    new_test_ext().execute_with(|| {
        let id = crate::Pallet::<Test>::get_new_deposit_id();
        assert!(id == 0);
        let id = crate::Pallet::<Test>::get_new_deposit_id();
        assert!(id == 1);
        let id = crate::Pallet::<Test>::get_new_deposit_id();
        assert_eq!(id, 2);
    });
}

#[test]
fn take_deposit_takes_deposit() {
    new_test_ext().execute_with(|| {
        let item = StorageItem::CrowdFund;
        let expected_deposit = MockDepositCalculator::calculate_deposit(item).unwrap();
        let alice_reserved_before = <Test as Config>::Currency::reserved_balance(&*ALICE);
        assert_ok!(crate::Pallet::<Test>::take_deposit(*ALICE, item));
        let alice_reserved_after = <Test as Config>::Currency::reserved_balance(&*ALICE);
        assert_eq!(
            alice_reserved_after - alice_reserved_before,
            expected_deposit,
            "Reserved balance should include the deposit."
        );
    });
}

#[test]
fn take_deposit_assert_last_event() {
    new_test_ext().execute_with(|| {
        let item = StorageItem::CrowdFund;
        let deposit = MockDepositCalculator::calculate_deposit(item).unwrap();
        assert_ok!(crate::Pallet::<Test>::take_deposit(*ALICE, item));
        System::assert_last_event(mock::RuntimeEvent::Deposits(
            crate::Event::<Test>::DepositTaken(0, deposit),
        ));
    });
}

#[test]
fn return_deposit_works() {
    new_test_ext().execute_with(|| {
        let item = StorageItem::CrowdFund;
        let alice_reserved_before = <Test as Config>::Currency::reserved_balance(&*ALICE);
        let deposit_id = crate::Pallet::<Test>::take_deposit(*ALICE, item).unwrap();
        assert_ok!(crate::Pallet::<Test>::return_deposit(deposit_id));
        let alice_reserved_after = <Test as Config>::Currency::reserved_balance(&*ALICE);
        assert_eq!(alice_reserved_before, alice_reserved_after);
    });
}

#[test]
fn return_deposit_assert_event() {
    new_test_ext().execute_with(|| {
        let item = StorageItem::CrowdFund;
        let deposit = MockDepositCalculator::calculate_deposit(item).unwrap();
        let deposit_id = crate::Pallet::<Test>::take_deposit(*ALICE, item).unwrap();
        assert_ok!(crate::Pallet::<Test>::return_deposit(deposit_id));
        System::assert_last_event(mock::RuntimeEvent::Deposits(
            crate::Event::<Test>::DepositReturned(0, deposit),
        ));
    });
}

#[test]
fn slash_deposit_works() {
    new_test_ext().execute_with(|| {
        let slash_account = <Test as Config>::DepositSlashAccount::get();
        assert_eq!(
            <Test as Config>::Currency::free_balance(&slash_account),
            0,
            "slash account should be empty to start"
        );
        let item = StorageItem::CrowdFund;
        let deposit = MockDepositCalculator::calculate_deposit(item).unwrap();
        let alice_reserved_before = <Test as Config>::Currency::reserved_balance(&*ALICE);
        let deposit_id = crate::Pallet::<Test>::take_deposit(*ALICE, item).unwrap();

        assert_ok!(Deposits::slash_reserve_deposit(deposit_id));
        let alice_reserved_after = <Test as Config>::Currency::reserved_balance(&*ALICE);
        let slash_account_balance_free = <Test as Config>::Currency::free_balance(&slash_account);

        assert_eq!(
            slash_account_balance_free, deposit,
            "slash_account's free balance should equal the deposit"
        );

        assert_eq!(
            alice_reserved_after - alice_reserved_before,
            deposit,
            "Alice's reserve balance should have been emptied"
        );
    });
}

#[test]
fn slash_deposit_assert_event() {
    new_test_ext().execute_with(|| {
        let item = StorageItem::CrowdFund;
        let deposit = MockDepositCalculator::calculate_deposit(item).unwrap();
        let deposit_id = crate::Pallet::<Test>::take_deposit(*ALICE, item).unwrap();

        assert_ok!(Deposits::slash_reserve_deposit(deposit_id));
        System::assert_last_event(mock::RuntimeEvent::Deposits(
            crate::Event::<Test>::DepositSlashed(0, deposit),
        ));
    });
}

#[test]
fn return_deposit_ignores_u32_max() {
    new_test_ext().execute_with(|| {
        // TODO:
    });
}
