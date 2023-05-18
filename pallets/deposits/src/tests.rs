use crate::{mock::*, Error, Event, traits::{DepositCalculator, DepositHandler}, Config, CurrentDeposits };
use frame_support::{assert_noop, assert_ok};
use common_types::CurrencyId;
use orml_traits::MultiReservableCurrency;

#[test]
fn take_deposit_works() {
    new_test_ext().execute_with(|| {
        let currency_id = CurrencyId::Native;
        let storage_item = StorageItem::Project;
        let deposit_id = DepositId::Project(0);
        let expected_deposit_taken = MockDepositCalculator::calculate_deposit(storage_item, currency_id);
        assert_ok!(Deposits::take_deposit(*ALICE, deposit_id, storage_item, currency_id));
        let reserved = <Test as Config>::MultiCurrency::reserved_balance(currency_id, &ALICE);
        assert_eq!(reserved, expected_deposit_taken);
        assert!(CurrentDeposits::<Test>::contains_key(deposit_id));
        assert!(!CurrentDeposits::<Test>::contains_key(DepositId::Project(1)));
        assert!(!CurrentDeposits::<Test>::contains_key(DepositId::Grant(0)));
    });
}


#[test]
fn take_deposit_id_already_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(Deposits::take_deposit(*ALICE, DepositId::Project(0), StorageItem::Project, CurrencyId::Native));
        assert_noop!(Deposits::take_deposit(*ALICE, DepositId::Project(0), StorageItem::Project, CurrencyId::Native), Error::<Test>::DepositAlreadyExists);
        assert_noop!(Deposits::take_deposit(*BOB, DepositId::Project(0), StorageItem::Project, CurrencyId::Native), Error::<Test>::DepositAlreadyExists);
    });
}

#[test]
fn reinstate_deposit_works() {
    new_test_ext().execute_with(|| {
        let deposit_id = DepositId::Project(0);
        assert_ok!(Deposits::take_deposit(*ALICE, deposit_id, StorageItem::Project, CurrencyId::Native));
        assert_ok!(Deposits::reinstate_deposit(deposit_id));
        let reserved: Balance = <Test as Config>::MultiCurrency::reserved_balance(CurrencyId::Native, &ALICE);
        assert_eq!(reserved, 0);
    });
}


#[test]
fn reinstate_deposit_removes_from_storage() {
    new_test_ext().execute_with(|| {
        let deposit_id = DepositId::Project(0);
        assert_ok!(Deposits::take_deposit(*ALICE, deposit_id, StorageItem::Project, CurrencyId::Native));
        assert_ok!(Deposits::reinstate_deposit(deposit_id));
        assert!(!CurrentDeposits::<Test>::contains_key(deposit_id));
    });
}


#[test]
fn reinstate_deposit_not_found() {
    new_test_ext().execute_with(|| {
        let deposit_id = DepositId::Project(0);
        assert_ok!(Deposits::take_deposit(*ALICE, deposit_id, StorageItem::Project, CurrencyId::Native));
        assert_ok!(Deposits::reinstate_deposit(deposit_id));
        assert_noop!(Deposits::reinstate_deposit(deposit_id), Error::<Test>::DepositDoesntExist);
    });
}