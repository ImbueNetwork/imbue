
use crate::mock::*;
use crate::pallet::*;
use crate::*;
use crate::traits::{DepositCalculator, DepositHandler};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use common_types::{CurrencyId, TreasuryOrigin};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};
use sp_runtime::traits::{Zero, One};

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
        let expected_deposit = MockDepositCalculator::calculate_deposit(item, CurrencyId::Native).unwrap();
        let alice_reserved_before = <Test as Config>::MultiCurrency::reserved_balance(CurrencyId::Native, &ALICE);
        let _ = crate::Pallet::<Test>::take_deposit(*ALICE, item, CurrencyId::Native);
        let alice_reserved_after = <Test as Config>::MultiCurrency::reserved_balance(CurrencyId::Native, &ALICE);
        assert_eq!(alice_reserved_after - alice_reserved_before, expected_deposit, "Reserved balance should include the deposit.");
    });
}

