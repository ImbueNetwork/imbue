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

use xcm::latest::{Junction, Junction::*, Junctions::*, MultiLocation, NetworkId, WeightLimit};

use common_runtime::{common_xcm::general_key, parachains};

use crate::kusama_test_net::{Development, Sibling, Kusama, TestNet, KusamaSender, KusamaReceiver, ImbueKusamaSender, ImbueKusamaReceiver, SiblingKusamaSender, SiblingKusamaReceiver};
use crate::setup::{
    development_account, kar_amount, karura_account, ksm_amount, native_amount, mgx_amount,
    sibling_account, PARA_ID_DEVELOPMENT, PARA_ID_SIBLING,
};
use common_runtime::Balance;
use common_types::{CurrencyId, FundingType, TreasuryOrigin};
use imbue_kusama_runtime::{
    AUsdPerSecond, Balances, CanonicalImbuePerSecond, KarPerSecond, KsmPerSecond, OrmlTokens,
    Runtime as R, RuntimeOrigin, XTokens, System,
};
use orml_traits::MultiCurrency;
use pallet_proposals::traits::RefundHandler;
use crate::constants::{kusama, SAFE_XCM_VERSION};
use xcm_emulator::{assert_expected_events, Parachain as Para};
use imbue_kusama_runtime::PolkadotXcm;

#[test]
fn test_xcm_refund_handler_to_kusama() {
    let treasury_origin = TreasuryOrigin::Kusama;
    let kusama_treasury_address =
        <R as pallet_proposals::Config>::RefundHandler::get_treasury_account_id(treasury_origin)
            .unwrap();
    let kusama_treasury_balance_before = Kusama::account_data_of(kusama_treasury_address.clone()).free;
    let transfer_amount: Balance = ksm_amount(10);
    Development::execute_with(|| {
        let ksm_balance = OrmlTokens::free_balance(CurrencyId::KSM, &ImbueKusamaReceiver::get().into());
        assert_eq!(ksm_balance, 0);
    });

    Kusama::execute_with(|| {
        assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
            kusama_runtime::RuntimeOrigin::signed(KusamaSender::get()),
            Box::new(Parachain(PARA_ID_DEVELOPMENT).into()),
            Box::new(
                Junction::AccountId32 {
                    network: Some(NetworkId::Kusama),
                    id: ImbueKusamaReceiver::get().into(),
                }
                .into()
            ),
            Box::new((Here, transfer_amount).into()),
            0
        ));
    });
    Development::execute_with(|| {
        let ksm_balance = OrmlTokens::free_balance(CurrencyId::KSM, &ImbueKusamaReceiver::get().into());
        assert!(ksm_balance > 0);
        assert_ok!(
            <R as pallet_proposals::Config>::RefundHandler::send_refund_message_to_treasury(
                ImbueKusamaReceiver::get().into(),
                ksm_balance,
                CurrencyId::KSM,
                FundingType::Grant(TreasuryOrigin::Kusama)
            )
        );
    });
    // TODO: Looks like the cumulus xcm simulator can't handle transfers other than native tokens. Investigate why we see successful sending of tokens but no horizontal message passing
    // let kusama_treasury_balance_after = Kusama::account_data_of(kusama_treasury_address.clone()).free;
    // assert!(kusama_treasury_balance_after > kusama_treasury_balance_before)
}

#[test]
fn transfer_ksm_to_relay_chain() {
    let transfer_amount: Balance = ksm_amount(10);
    let bob_initial_balance = ksm_amount(1_000);
    let kusama_receiver_balance_before = Kusama::account_data_of(KusamaReceiver::get()).free;
    Kusama::execute_with(|| {
        assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
            kusama_runtime::RuntimeOrigin::signed(KusamaSender::get().into()),
            Box::new(Parachain(PARA_ID_DEVELOPMENT).into()),
            Box::new(
                Junction::AccountId32 {
                    network: Some(NetworkId::Kusama),
                    id: ImbueKusamaSender::get().clone().into(),
                }
                .into()
            ),
            Box::new((Here, bob_initial_balance).into()),
            0,
        ));
    });

    Development::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ImbueKusamaSender::get().clone().into()),
            CurrencyId::KSM,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X1(Junction::AccountId32 {
                        id: KusamaReceiver::get().clone().into(),
                        network: Some(NetworkId::Kusama),
                    })
                )
                .into()
            ),
            xcm_emulator::Limited(4_000_000_000.into())
        ));
    });

    // TODO: Looks like the cumulus xcm simulator can't handle transfers other than native tokens. Investigate why we see successful sending of tokens but no horizontal message passing
    // Kusama::execute_with(|| {
    //     let kusama_receiver_balance_after = Balances::free_balance(&KusamaReceiver::get().into());
    //     let test = System::events();
    //     assert!(kusama_receiver_balance_after > kusama_receiver_balance_before);
    // });
}

#[test]
fn transfer_from_relay_chain() {
    let transfer_amount: Balance = ksm_amount(1);
    Development::execute_with(|| {
        assert_eq!(
            OrmlTokens::free_balance(CurrencyId::KSM, &ImbueKusamaReceiver::get().into()),
            0
        );
    });

    Kusama::execute_with(|| {
        assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
            kusama_runtime::RuntimeOrigin::signed(KusamaSender::get().into()),
            Box::new(Parachain(PARA_ID_DEVELOPMENT).into()),
            Box::new(
                Junction::AccountId32 {
                    network: Some(NetworkId::Kusama),
                    id: ImbueKusamaReceiver::get().into(),
                }
                .into()
            ),
            Box::new((Here, transfer_amount.clone()).into()),
            0,
        ));
    });

    Development::execute_with(|| {
        let para_receiver_balance_after = OrmlTokens::free_balance(CurrencyId::KSM, &ImbueKusamaReceiver::get().into());
        assert!(para_receiver_balance_after > 0);
    });
}

#[test]
fn transfer_native_to_sibling() {
    Development::execute_with(|| {
        assert_ok!(PolkadotXcm::force_xcm_version(RuntimeOrigin::root(),
             Box::new(MultiLocation::new(
                    1,
                    X1(
                        Parachain(PARA_ID_SIBLING)
                    ),
            )),SAFE_XCM_VERSION));
    });
    let transfer_amount: Balance = native_amount(10);
    let sibling_balance_before: Balance = Sibling::account_data_of(SiblingKusamaReceiver::get().into()).free;
    Development::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ImbueKusamaSender::get().into()),
            CurrencyId::Native,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 {
                            network: Some(NetworkId::Kusama),
                            id: SiblingKusamaReceiver::get().into(),
                        }
                    )
                )
                .into()
            ),
            xcm_emulator::Limited(4_000_000_000.into())
        ));
    });

    let sibling_balance_after = Sibling::account_data_of(SiblingKusamaReceiver::get().into()).free;
    assert!(sibling_balance_after > sibling_balance_before);
}

#[test]
fn transfer_mgx_to_sibling() {
    let transfer_amount = mgx_amount(10);
    Development::execute_with(|| {
        assert_ok!(OrmlTokens::deposit(
            CurrencyId::MGX,
            &ImbueKusamaSender::get().into(),
            transfer_amount.saturating_mul(2)
        ));
        assert_ok!(PolkadotXcm::force_xcm_version(RuntimeOrigin::root(),
             Box::new(MultiLocation::new(
                    1,
                    X1(
                        Parachain(PARA_ID_SIBLING)
                    ),
            )),SAFE_XCM_VERSION));
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ImbueKusamaSender::get().into()),
            CurrencyId::MGX,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 {
                            network: Some(NetworkId::Kusama),
                            id: SiblingKusamaReceiver::get().into(),
                        }
                    )
                )
                .into()
            ),
            xcm_emulator::Limited(8_000_000_000.into())
        ));
    });

    // TODO: Looks like the cumulus xcm simulator can't handle transfers other than native tokens. Investigate why we see successful sending of tokens but no horizontal message passing
    //
    // Sibling::execute_with(|| {
    //     let mgx_balance = OrmlTokens::free_balance(CurrencyId::MGX, &SiblingKusamaReceiver::get().into()),
    //     assert!(mgx_balance > 0);
    //     let test = System::events();
    //     let blah =1 ;
    // });
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
    let (_asset, fee, _) = CanonicalImbuePerSecond::get();
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee.div_euclid(10_000) * 8
}

// The fee associated with transferring AUSD tokens
// fn ausd_fee() -> Balance {
//     let (_asset, fee, _) = AUsdPerSecond::get();
//     // NOTE: it is possible that in different machines this value may differ. We shall see.
//     fee.div_euclid(10_000) * 8
// }
//
// // The fee associated with transferring AUSD tokens
// fn kar_fee() -> Balance {
//     let (_asset, fee, _) = KarPerSecond::get();
//     // NOTE: it is possible that in different machines this value may differ. We shall see.
//     fee.div_euclid(10_000) * 8
// }
//
//
