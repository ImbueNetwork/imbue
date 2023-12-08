use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use test_utils::*;

#[test]
fn set_foreign_asset_signer_check_permission_for_edit() {
    build_test_externality().execute_with(|| {
        assert_ok!(Proposals::set_foreign_asset_signer(
            RuntimeOrigin::root(),
            ALICE
        ));
        assert_eq!(
            ForeignCurrencySigner::<Test>::get().unwrap(),
            ALICE,
            "Alice should have been set as signer."
        );
        assert_ok!(Proposals::set_foreign_asset_signer(
            RuntimeOrigin::root(),
            BOB
        ));
        assert_eq!(
            ForeignCurrencySigner::<Test>::get().unwrap(),
            BOB,
            "Bob should be set as signer."
        );
        assert_noop!(
            Proposals::set_foreign_asset_signer(RuntimeOrigin::signed(BOB), ALICE),
            BadOrigin
        );
    })
}

#[test]
fn foreign_asset_signer_can_mint() {
    build_test_externality().execute_with(|| {
        let currency_id = CurrencyId::ForeignAsset(10);
        let beneficiary = BOB;
        let amount = 92839572;
        let _ = Proposals::set_foreign_asset_signer(RuntimeOrigin::root(), ALICE);
        let asset_signer = ForeignCurrencySigner::<Test>::get().unwrap();
        assert_eq!(Tokens::free_balance(currency_id, &BOB), 0);
        assert_ok!(Proposals::mint_offchain_assets(
            RuntimeOrigin::signed(asset_signer),
            beneficiary,
            currency_id,
            amount
        ));
        assert_eq!(Tokens::free_balance(currency_id, &BOB), amount);
    })
}

#[test]
fn non_foreign_asset_signer_cannot_mint() {
    build_test_externality().execute_with(|| {
        let currency_id = CurrencyId::ForeignAsset(10);
        let beneficiary = BOB;
        let amount = 92839572;
        let _ = Proposals::set_foreign_asset_signer(RuntimeOrigin::root(), ALICE);
        let asset_signer = ForeignCurrencySigner::<Test>::get().unwrap();

        assert_noop!(
            Proposals::mint_offchain_assets(
                RuntimeOrigin::signed(BOB),
                beneficiary,
                currency_id,
                amount
            ),
            Error::<Test>::RequireForeignAssetSigner
        );
        assert_noop!(
            Proposals::mint_offchain_assets(
                RuntimeOrigin::signed(CHARLIE),
                beneficiary,
                currency_id,
                amount
            ),
            Error::<Test>::RequireForeignAssetSigner
        );
    })
}
