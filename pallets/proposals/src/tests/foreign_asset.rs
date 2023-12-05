#[test]
fn set_foreign_asset_signer_check_permission_for_edit() {
    build_test_externality().execute_with(|| {
        Proposals::set_foreign_asset_signer(RuntimeOrigin::root(), ALICE);
        assert!(ForeignCurrencySigner::<Test>::get().unwrap(), ALICE);
        Proposals::set_foreign_asset_signer(RuntimeOrigin::root(), BOB);
        assert!(ForeignCurrencySigner::<Test>::get().unwrap(), BOB);

        assert_noop!(Proposals::set_foreign_asset_signer(RuntimeOrigin::signed(BOB), ALICE), BadOrigin);
    })
}


#[test]
fn foreign_asset_signer_can_mint() {
    build_test_externality().execute_with(|| {

    })
}