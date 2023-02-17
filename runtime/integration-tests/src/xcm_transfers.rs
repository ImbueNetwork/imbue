// Copyright 2022 Imbue Network (imbue.network).
// This file is part of Imbue chain project.

// Imbue is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).

// Imbue is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
use frame_support::assert_ok;
use xcm_emulator::TestExt;

use xcm::latest::{Junction, Junction::*, Junctions::*, MultiLocation, NetworkId};

use common_runtime::{common_xcm::general_key, parachains};

use crate::kusama_test_net::{Development, Karura, KusamaNet, Sibling, TestNet};
use crate::setup::{
    ausd_amount, development_account, kar_amount, karura_account, ksm_amount, native_amount,
    sibling_account, ALICE, BOB, PARA_ID_DEVELOPMENT, PARA_ID_SIBLING,
};
use common_runtime::Balance;
use common_types::CurrencyId;
use imbue_kusama_runtime::{
    AUsdPerSecond, Balances, CanonicalImbuePerSecond, KarPerSecond, KsmPerSecond, RuntimeOrigin,
    OrmlTokens, XTokens,
};
use orml_traits::MultiCurrency;

#[test]
fn transfer_native_to_sibling() {
    TestNet::reset();

    let alice_initial_balance = native_amount(10);
    let bob_initial_balance = native_amount(10);
    let transfer_amount = native_amount(1);

    Development::execute_with(|| {
        assert_eq!(Balances::free_balance(&ALICE.into()), alice_initial_balance);
        assert_eq!(Balances::free_balance(&sibling_account()), 0);
    });

    Sibling::execute_with(|| {
        assert_eq!(Balances::free_balance(&BOB.into()), bob_initial_balance);
    });

    Development::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE.into()),
            CurrencyId::Native,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 {
                            network: NetworkId::Any,
                            id: BOB.into(),
                        }
                    )
                )
                .into()
            ),
            xcm_emulator::Limited(1_000_000_000),
        ));

        // Confirm that Alice's balance is initial balance - amount transferred
        assert_eq!(
            Balances::free_balance(&ALICE.into()),
            alice_initial_balance - transfer_amount
        );

        // Verify that the amount transferred is now part of the sibling account here
        assert_eq!(Balances::free_balance(&sibling_account()), transfer_amount);
    });

    Sibling::execute_with(|| {
        // Verify that BOB now has initial balance + amount transferred - fee
        assert_eq!(
            Balances::free_balance(&BOB.into()),
            bob_initial_balance + transfer_amount - native_fee(),
        );
    });
}

#[test]
fn transfer_ausd_to_development() {
    TestNet::reset();

    let alice_initial_balance = ausd_amount(10);
    let bob_initial_balance = ausd_amount(10);
    let transfer_amount = ausd_amount(7);

    Karura::execute_with(|| {
        assert_ok!(OrmlTokens::deposit(
            CurrencyId::AUSD,
            &ALICE.into(),
            alice_initial_balance
        ));

        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::AUSD, &development_account()),
            0
        );
    });

    Development::execute_with(|| {
        assert_ok!(OrmlTokens::deposit(
            CurrencyId::AUSD,
            &BOB.into(),
            bob_initial_balance
        ));
        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::AUSD, &BOB.into()),
            bob_initial_balance,
        );

        assert_ok!(OrmlTokens::deposit(
            CurrencyId::AUSD,
            &karura_account().into(),
            bob_initial_balance
        ));
    });

    Karura::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE.into()),
            CurrencyId::AUSD,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_DEVELOPMENT),
                        Junction::AccountId32 {
                            network: NetworkId::Any,
                            id: BOB.into(),
                        }
                    )
                )
                .into()
            ),
            xcm_emulator::Limited(8_000_000_000),
        ));

        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::AUSD, &ALICE.into()),
            alice_initial_balance - transfer_amount
        );

        // Verify that the amount transferred is now part of the development account here
        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::AUSD, &development_account()),
            transfer_amount
        );
    });

    Development::execute_with(|| {
        // Verify that BOB now has initial balance + amount transferred - fee
        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::AUSD, &BOB.into()),
            bob_initial_balance + transfer_amount - ausd_fee()
        );
    });
}

#[test]
fn transfer_kar_to_development() {
    TestNet::reset();

    let alice_initial_balance = kar_amount(10);
    let bob_initial_balance = kar_amount(10);
    let transfer_amount = kar_amount(7);

    Karura::execute_with(|| {
        assert_ok!(OrmlTokens::deposit(
            CurrencyId::KAR,
            &ALICE.into(),
            alice_initial_balance
        ));

        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::KAR, &development_account()),
            0
        );
    });

    Development::execute_with(|| {
        assert_ok!(OrmlTokens::deposit(
            CurrencyId::KAR,
            &BOB.into(),
            bob_initial_balance
        ));
        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::KAR, &BOB.into()),
            bob_initial_balance,
        );

        assert_ok!(OrmlTokens::deposit(
            CurrencyId::AUSD,
            &karura_account().into(),
            bob_initial_balance
        ));
    });

    Karura::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE.into()),
            CurrencyId::KAR,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_DEVELOPMENT),
                        Junction::AccountId32 {
                            network: NetworkId::Any,
                            id: BOB.into(),
                        }
                    )
                )
                .into()
            ),
            xcm_emulator::Limited(8_000_000_000),
        ));

        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::KAR, &ALICE.into()),
            alice_initial_balance - transfer_amount
        );

        // Verify that the amount transferred is now part of the development account here
        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::KAR, &development_account()),
            transfer_amount
        );
    });

    Development::execute_with(|| {
        // Verify that BOB now has initial balance + amount transferred - fee
        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::KAR, &BOB.into()),
            bob_initial_balance + transfer_amount - kar_fee()
        );
    });
}

#[test]
fn transfer_from_relay_chain() {
    let transfer_amount: Balance = ksm_amount(1);

    KusamaNet::execute_with(|| {
        assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
            kusama_runtime::RuntimeOrigin::signed(ALICE.into()),
            Box::new(Parachain(PARA_ID_DEVELOPMENT).into().into()),
            Box::new(
                Junction::AccountId32 {
                    network: NetworkId::Any,
                    id: BOB,
                }
                .into()
                .into()
            ),
            Box::new((Here, transfer_amount).into()),
            0
        ));
    });

    Development::execute_with(|| {
        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::KSM, &BOB.into()),
            transfer_amount - ksm_fee()
        );
    });
}

#[test]
fn transfer_ksm_to_relay_chain() {
    Development::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE.into()),
            CurrencyId::KSM,
            ksm_amount(1),
            Box::new(
                MultiLocation::new(
                    1,
                    X1(Junction::AccountId32 {
                        id: BOB,
                        network: NetworkId::Any,
                    })
                )
                .into()
            ),
            xcm_emulator::Limited(4_000_000_000)
        ));
    });

    KusamaNet::execute_with(|| {
        assert_eq!(
            kusama_runtime::Balances::free_balance(&BOB.into()),
            999895428355
        );
    });
}

#[test]
fn currency_id_convert_imbu() {
    use imbue_kusama_runtime::CurrencyIdConvert;
    use sp_runtime::traits::Convert as C2;

    let imbu_location: MultiLocation = MultiLocation::new(
        1,
        X2(
            Parachain(parachains::kusama::imbue::ID),
            general_key(parachains::kusama::imbue::IMBU_KEY),
        ),
    );

    assert_eq!(
        CurrencyIdConvert::convert(imbu_location.clone()),
        Some(CurrencyId::Native),
    );

    let imbu_location_2: MultiLocation =
        MultiLocation::new(0, X1(general_key(parachains::kusama::imbue::IMBU_KEY)));

    assert_eq!(
        CurrencyIdConvert::convert(imbu_location_2.clone()),
        Some(CurrencyId::Native),
    );
}

// The fee associated with transferring Native tokens
fn native_fee() -> Balance {
    let (_asset, fee) = CanonicalImbuePerSecond::get();
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee.div_euclid(10_000) * 8
}

// The fee associated with transferring AUSD tokens
fn ausd_fee() -> Balance {
    let (_asset, fee) = AUsdPerSecond::get();
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee.div_euclid(10_000) * 8
}

// The fee associated with transferring AUSD tokens
fn kar_fee() -> Balance {
    let (_asset, fee) = KarPerSecond::get();
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee.div_euclid(10_000) * 8
}

// The fee associated with transferring KSM tokens
fn ksm_fee() -> Balance {
    let (_asset, fee) = KsmPerSecond::get();
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee.div_euclid(10_000) * 8
}
