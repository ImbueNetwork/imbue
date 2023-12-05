use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use test_utils::*;

#[test]
fn set_foreign_asset_signer_check_permission_for_edit() {
    build_test_externality().execute_with(|| {
        assert_ok!(Proposals::set_foreign_asset_signer(RuntimeOrigin::root(), ALICE));
        assert!(ForeignCurrencySigner::<Test>::get().unwrap(), ALICE);
        assert_ok!(Proposals::set_foreign_asset_signer(RuntimeOrigin::root(), BOB));
        assert!(ForeignCurrencySigner::<Test>::get().unwrap(), BOB);
        assert_noop!(Proposals::set_foreign_asset_signer(RuntimeOrigin::signed(BOB), ALICE), BadOrigin);
    })
}

#[test]
fn foreign_asset_signer_can_mint() {
    build_test_externality().execute_with(|| {
        let currency_id = ForeignAsset(10);
        let beneficiary = BOB;
        let amount 92839572;
        Proposals::set_foreign_asset_signer(RuntimeOrigin::root(), ALICE);
        let asset_signer = ForeignCurrencySigner::<Test>::get().unwrap();
        assert_eq!(
            Tokens::free_balance(currency_id, &BOB),
            0
        );
        assert_ok(Proposals::mint_offchain_assets(RuntimeOrigin::signed(asset_signer), beneficiary, currency_id, amount));
        assert_eq!(
            Tokens::free_balance(currency_id, &BOB),
            amount
        );
    })
}