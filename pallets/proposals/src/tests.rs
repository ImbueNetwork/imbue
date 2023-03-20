// use crate as proposals;
// use crate::mock::*;
//
// use crate::*;
// use common_types::CurrencyId;
// use frame_support::{
//     assert_noop, assert_ok, bounded_btree_map, bounded_vec,
//     dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo},
// };
// use sp_core::sr25519;
// use sp_std::vec::Vec;
//
// #[test]
// fn create_a_test_project() {
//     build_test_externality().execute_with(|| {
//         assert_ok!(create_projects_with_inputs(
//             "Imbue's Awesome Initiative",
//             "Imbue Logo",
//             "This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.",
//             "https://imbue.network",
//             100_000u64
//         ));
//     });
// }
//
// #[test]
// fn create_a_test_project_with_less_than_100_percent() {
//     build_test_externality().execute_with(|| {
//         let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//         assert_noop!(
//         Proposals::create_project(
//             RuntimeOrigin::signed(alice),
//             //project name
//             b"Imbue's Awesome Initiative".to_vec().try_into().expect("input should be of decent length"),
//             //project logo
//             b"Imbue Logo".to_vec().try_into().expect("input should be of decent length"),
//             //project description
//             b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.".to_vec().try_into().expect("input should be of decent length"),
//             //website
//             b"https://imbue.network".to_vec().try_into().expect("input should be of decent length"),
//             //milestone
//             bounded_vec![ProposedMilestone {
//                 name: bounded_vec![], percentage_to_unlock: 99
//             }],
//             //funds required
//             1000000u64,
//             CurrencyId::Native
//         ),DispatchErrorWithPostInfo {
//             post_info: PostDispatchInfo {
//                 actual_weight: None,
//                 pays_fee: Pays::Yes,
//             },
//             error: Error::<Test>::MilestonesTotalPercentageMustEqual100.into()
//         });
//     });
// }
//
// #[test]
// fn create_a_test_project_with_no_name() {
//     build_test_externality().execute_with(|| {
//         assert_noop!(
//             create_projects_with_inputs("", "logo", "description", "website", 100_000u64),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::ProjectNameIsMandatory.into()
//             }
//         );
//     });
// }
// #[test]
// fn create_a_test_project_with_no_logo() {
//     build_test_externality().execute_with(|| {
//         assert_noop!(
//             create_projects_with_inputs("name", "", "description", "website", 100_000u64),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::LogoIsMandatory.into()
//             }
//         );
//     });
// }
// #[test]
// fn create_a_test_project_with_no_description() {
//     build_test_externality().execute_with(|| {
//         assert_noop!(
//             create_projects_with_inputs("name", "logo", "", "website", 100_000u64),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::ProjectDescriptionIsMandatory.into()
//             }
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_with_no_website() {
//     build_test_externality().execute_with(|| {
//         assert_noop!(
//             create_projects_with_inputs("name", "logo", "description", "", 100_000u64),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::WebsiteURLIsMandatory.into()
//             }
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_add_whitelist() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let max_cap = 1_000_000u64;
//     let project_key = 0;
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//         let whitelist = bounded_btree_map!(alice => max_cap);
//
//         Proposals::add_project_whitelist(RuntimeOrigin::signed(alice), project_key, whitelist)
//             .unwrap();
//
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::WhitelistAdded(0, 1))
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_add_whitelist_from_non_initiator_fail() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let max_cap = 1000000u64;
//     let project_key = 0;
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//         let whitelist = bounded_btree_map!(alice => max_cap);
//
//         assert_noop!(
//             Proposals::add_project_whitelist(RuntimeOrigin::signed(bob), project_key, whitelist),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::UserIsNotInitiator.into()
//             }
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_remove_whitelist() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//         Proposals::remove_project_whitelist(RuntimeOrigin::signed(alice), 0).unwrap();
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::WhitelistRemoved(0, 1))
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_schedule_round() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//
//         assert_ok!(Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             //Project key starts with 0 for the first project submitted to the chain
//             bounded_vec![0],
//             RoundType::ContributionRound,
//         ));
//     });
// }
//
// #[test]
// fn schedule_round_invalid_project_key() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//
//         assert_noop!(
//             Proposals::schedule_round(
//                 RuntimeOrigin::root(),
//                 System::block_number(),
//                 System::block_number() + 1,
//                 //Project key starts with 0 for the first project submitted to the chain
//                 bounded_vec![1],
//                 RoundType::ContributionRound
//             ),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::ProjectDoesNotExist.into()
//             }
//         );
//     });
// }
//
// #[test]
// fn schedule_round_invalid_end_block_no() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//
//         assert_noop!(
//             Proposals::schedule_round(
//                 RuntimeOrigin::root(),
//                 System::block_number() + 6000,
//                 System::block_number() + 3000,
//                 //Project key starts with 0 for the first project submitted to the chain
//                 bounded_vec![1],
//                 RoundType::ContributionRound
//             ),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::EndTooEarly.into()
//             }
//         );
//     });
// }
//
// #[test]
// fn cancel_round_no_active_round() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//
//         let _ = Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 3000,
//             System::block_number() + 6000,
//             bounded_vec![0],
//             RoundType::ContributionRound,
//         );
//
//         assert_noop!(
//             Proposals::cancel_round(RuntimeOrigin::root(), 0),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::NoActiveRound.into()
//             }
//         );
//     });
// }
//
// #[test]
// fn test_funding_round_is_created_on_schedule_round() {
//     let project_keys: BoundedProjectKeys = bounded_vec![0u32];
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     //create_project extrinsic
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 2,
//             project_keys.clone(),
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let exp_fundingroundcreated_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//
//         assert_eq!(
//             exp_fundingroundcreated_event,
//             mock::RuntimeEvent::from(proposals::Event::FundingRoundCreated(
//                 1,
//                 project_keys.to_vec()
//             ))
//         );
//     });
// }
//
// #[test]
// fn cancel_round() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     //create_project extrinsic
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         //schedule_round extrinsic
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 2,
//             project_keys.clone(),
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let round_index = 1;
//
//         //cancel_round extrinsic
//         assert_ok!(<proposals::Pallet<Test>>::cancel_round(
//             RuntimeOrigin::root(),
//             round_index
//         ));
//
//         let exp_roundcancelled_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             exp_roundcancelled_event,
//             mock::RuntimeEvent::from(proposals::Event::RoundCancelled(1))
//         );
//     });
// }
//
// #[test]
// fn test_cancelling_started_round() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let round_key = 1;
//
//         assert_noop!(
//             Proposals::cancel_round(RuntimeOrigin::root(), round_key),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::RoundStarted.into(),
//             }
//         );
//     });
// }
//
// #[test]
// //only user with root privilege can cancel the round
// fn test_cancelling_round_without_root_privilege() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//         let round_key = 1;
//         assert_noop!(
//             Proposals::cancel_round(RuntimeOrigin::signed(alice), round_key),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: DispatchError::BadOrigin,
//             }
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_schedule_round_and_contribute() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         //create_project extrinsic
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         let project_key: u32 = 0;
//         let contribution_amount = 2000u64;
//
//         //schedule_round extrinsic
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 10,
//             //Project key starts with 0 for the first project submitted to the chain
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//         let additional_amount = 10_000;
//
//         let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);
//
//         run_to_block(4);
//         //contribute extrinsic
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//
//         //contribute success RuntimeEvent
//         let exp_contributedtoproject_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             exp_contributedtoproject_event,
//             mock::RuntimeEvent::from(proposals::Event::ContributeSucceeded(
//                 alice,
//                 project_key,
//                 contribution_amount,
//                 CurrencyId::Native,
//                 4
//             ))
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_schedule_round_and_add_whitelist_with_cap_and_contribute() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         //create_project extrinsic
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         let project_key: u32 = 0;
//         let contribution_amount = 2000u64;
//         let max_cap = 1000000u64;
//
//         let whitelist = bounded_btree_map!(alice => max_cap);
//         Proposals::add_project_whitelist(RuntimeOrigin::signed(alice), project_key, whitelist)
//             .unwrap();
//
//         //schedule_round extrinsic
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 10,
//             //Project key starts with 0 for the first project submitted to the chain
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//         let additional_amount = contribution_amount;
//
//         let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);
//
//         run_to_block(4);
//
//         //contribute extrinsic
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//
//         //contribute success RuntimeEvent
//         let exp_contributedtoproject_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             exp_contributedtoproject_event,
//             mock::RuntimeEvent::from(proposals::Event::ContributeSucceeded(
//                 alice,
//                 project_key,
//                 contribution_amount,
//                 CurrencyId::Native,
//                 4
//             ))
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_schedule_round_and_add_whitelist_with_unlimited_cap_and_contribute() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         //create_project extrinsic
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         let project_key: u32 = 0;
//         let contribution_amount = 2000u64;
//         let max_cap = 0u64;
//
//         let whitelist = bounded_btree_map!(alice => max_cap);
//         Proposals::add_project_whitelist(RuntimeOrigin::signed(alice), project_key, whitelist)
//             .unwrap();
//
//         //schedule_round extrinsic
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 10,
//             //Project key starts with 0 for the first project submitted to the chain
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//         let additional_amount = contribution_amount;
//
//         let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);
//
//         run_to_block(4);
//
//         //contribute extrinsic
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//
//         //contribute success RuntimeEvent
//         let exp_contributedtoproject_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             exp_contributedtoproject_event,
//             mock::RuntimeEvent::from(proposals::Event::ContributeSucceeded(
//                 alice,
//                 project_key,
//                 contribution_amount,
//                 CurrencyId::Native,
//                 4
//             ))
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_schedule_round_and_add_whitelist_and_contribute_over_capfail() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         //create_project extrinsic
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         let project_key: u32 = 0;
//         let contribution_amount = 60_000u64;
//         let max_cap = 100_000u64;
//
//         let whitelist = bounded_btree_map!(alice => max_cap);
//         Proposals::add_project_whitelist(RuntimeOrigin::signed(alice), project_key, whitelist)
//             .unwrap();
//
//         //schedule_round extrinsic
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 10,
//             //Project key starts with 0 for the first project submitted to the chain
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//         let alice_balance = 100_000_000u64;
//         let _ = Currencies::deposit(CurrencyId::Native, &alice, alice_balance);
//
//         run_to_block(4);
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//
//         assert_noop!(
//             Proposals::contribute(
//                 RuntimeOrigin::signed(alice),
//                 None,
//                 project_key,
//                 contribution_amount
//             ),
//             //approve project
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::ContributionMustBeLowerThanMaxCap.into()
//             }
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_schedule_round_and_contribute_and_approve() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         //create_project extrinsic
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         let project_key = 0;
//         let contribution_amount = 1000000u64;
//
//         //schedule_round extrinsic
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 10,
//             //Project key starts with 0 for the first project submitted to the chain
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//         let additional_amount = contribution_amount;
//         let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);
//
//         run_to_block(4);
//         //contribute extrinsic
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//
//         let project_key = 0;
//         //approve project
//         Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();
//
//         //approve RuntimeEvent
//         let exp_approvedproject_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             exp_approvedproject_event,
//             mock::RuntimeEvent::from(proposals::Event::ProjectApproved(1, project_key))
//         );
//     });
// }
//
// #[test]
// //negative test case - Approve fails because contribution amount has not met the project required funds
// fn create_a_test_project_and_schedule_round_and_contribute_and_approvefail() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         //create_project extrinsic
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         let project_key = 0;
//         let contribution_amount = 100000u64;
//
//         //schedule_round extrinsic
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 10,
//             //Project key starts with 0 for the first project submitted to the chain
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//         let additional_amount = contribution_amount;
//         let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);
//
//         run_to_block(4);
//         //contribute extrinsic
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//
//         assert_noop!(
//             //approve project
//             Proposals::approve(RuntimeOrigin::root(), None, project_key, None),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::RoundNotEnded.into()
//             }
//         );
//     });
// }
//
// #[test]
// fn test_submit_milestone() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//     let voting_round_key = 2;
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project(alice);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let value = 100u64;
//         Proposals::contribute(RuntimeOrigin::signed(bob), None, project_key, value).unwrap();
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(0);
//
//         run_to_block(3);
//
//         //Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();
//
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             0
//         ));
//
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::VotingRoundCreated(
//                 voting_round_key,
//                 vec![project_key]
//             ))
//         );
//     });
// }
//
// #[test]
// //negative test case - cannot submit milestones for unapproved projects
// fn test_submit_milestone_without_approval() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project(alice);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let value = 100u64;
//         assert_ok!(Proposals::contribute(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key,
//             value
//         ));
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(0);
//
//         run_to_block(3);
//
//         assert_noop!(
//             Proposals::submit_milestone(RuntimeOrigin::signed(alice), project_key, 0),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::OnlyApprovedProjectsCanSubmitMilestones.into(),
//             }
//         );
//     });
// }
//
// #[test]
// fn test_voting_on_a_milestone() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//     let milestone1_key = 0;
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project(alice);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let value = 100u64;
//         Proposals::contribute(RuntimeOrigin::signed(bob), None, project_key, value).unwrap();
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(0);
//
//         run_to_block(3);
//
//         assert_ok!(Proposals::approve(
//             RuntimeOrigin::root(),
//             None,
//             project_key,
//             None
//         ));
//
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             0
//         ));
//
//         run_to_block(5);
//         assert_ok!(Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(bob),
//             project_key,
//             milestone1_key,
//             None,
//             true
//         ));
//
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::VoteComplete(bob, 0, 0, true, 5))
//         );
//     });
// }
//
// #[test]
// //voting on cancelled round should throw error
// fn test_voting_on_a_cancelled_round() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//     let round_key = 1;
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project(alice);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![project_key];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 2,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         assert_ok!(<proposals::Pallet<Test>>::cancel_round(
//             RuntimeOrigin::root(),
//             round_key
//         ));
//
//         run_to_block(5);
//
//         // A strange test as voting on a milestone is not permitted during a contribution round, only a voting round.
//         // Todo:? test that contribution is not allowed after the round is cancelled.
//         let milestone_key = 0;
//         assert_noop!(
//             Proposals::vote_on_milestone(
//                 RuntimeOrigin::signed(bob),
//                 project_key,
//                 milestone_key,
//                 None,
//                 true
//             ),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::InvalidRoundType.into(),
//             }
//         );
//
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::RoundCancelled(round_key))
//         );
//     });
// }
//
// #[test]
// //negative test case where the project creator tries to finalize milestone without getting the vote on that milestone
// fn test_finalize_a_milestone_without_voting() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//     let milestone1_key = 0;
//     let milestone2_key = 1;
//     let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
//     let milestone1: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 1"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 20,
//     };
//     let milestone2: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 2"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 30,
//     };
//
//     let milestone3: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 3"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 50,
//     };
//     proposed_milestones.push(milestone1);
//     proposed_milestones.push(milestone2);
//     proposed_milestones.push(milestone3);
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project_multiple_milestones(alice, proposed_milestones);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let value = 100u64;
//         Proposals::contribute(RuntimeOrigin::signed(bob), None, project_key, value).unwrap();
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(0);
//         let _ = milestone_index.try_push(1);
//
//         run_to_block(3);
//
//         Proposals::approve(
//             RuntimeOrigin::root(),
//             None,
//             project_key,
//             Some(milestone_index),
//         )
//         .unwrap();
//
//         // Test you can submit a milestone whenever.
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             milestone1_key
//         ));
//
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             milestone2_key
//         ));
//
//         run_to_block(5);
//         assert_ok!(Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(bob),
//             project_key,
//             milestone1_key,
//             None,
//             true
//         ));
//
//         //this works as the voting has been done for this milestone
//         assert_ok!(Proposals::finalise_milestone_voting(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             0
//         ));
//
//         assert_noop!(
//             Proposals::finalise_milestone_voting(RuntimeOrigin::signed(alice), project_key, 1),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::MilestoneVotingNotComplete.into(),
//             }
//         );
//     });
// }
//
// #[test]
// fn test_project_initiator_cannot_withdraw_if_majority_vote_against() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
//     let additional_amount = 10_000_000u64;
//
//     let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
//
//     let milestone1: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 1"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 20,
//     };
//     let milestone2: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 2"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 30,
//     };
//
//     let milestone3: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 3"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 50,
//     };
//     proposed_milestones.push(milestone1);
//     proposed_milestones.push(milestone2);
//     proposed_milestones.push(milestone3);
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         let _ = Currencies::deposit(CurrencyId::Native, &charlie, additional_amount);
//         create_project_multiple_milestones(alice, proposed_milestones);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let bob_contribution = 200_000u64;
//         assert_ok!(Proposals::contribute(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key,
//             bob_contribution
//         ));
//
//         // Second contribution to give Bob majority
//         let bob_second_contribution = 400_000u64;
//         assert_ok!(Proposals::contribute(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key,
//             bob_second_contribution
//         ));
//
//         let charlie_contribution = 500_000u64;
//         assert_ok!(Proposals::contribute(
//             RuntimeOrigin::signed(charlie),
//             None,
//             project_key,
//             charlie_contribution
//         ));
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(0);
//         let _ = milestone_index.try_push(1);
//
//         run_to_block(3);
//
//         assert_ok!(Proposals::approve(
//             RuntimeOrigin::root(),
//             None,
//             project_key,
//             None
//         ));
//
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             0
//         ));
//
//         run_to_block(5);
//         let milestone_key = 0;
//         //Bob voting on the submitted milestone
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(bob),
//             project_key,
//             milestone_key,
//             None,
//             false,
//         )
//         .ok();
//
//         //Charlie voting on the submitted milestone
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(charlie),
//             project_key,
//             milestone_key,
//             None,
//             true,
//         )
//         .ok();
//
//         assert_ok!(Proposals::finalise_milestone_voting(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             0
//         ));
//
//         assert_noop!(
//             Proposals::withdraw(RuntimeOrigin::signed(alice), project_key),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::NoAvailableFundsToWithdraw.into(),
//             }
//         );
//     })
// }
//
// #[test]
// fn test_project_initiator_can_withdraw_only_the_percentage_milestone_completed() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
//     let additional_amount = 10000000u64;
//     let required_funds = 1000000u64;
//     let milestone1_key = 0;
//     let milestone2_key = 1;
//     let milestone3_key = 2;
//
//     let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
//
//     let milestone1: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 1"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 20,
//     };
//     let milestone2: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 2"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 30,
//     };
//
//     let milestone3: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 3"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 50,
//     };
//     proposed_milestones.push(milestone1);
//     proposed_milestones.push(milestone2);
//     proposed_milestones.push(milestone3);
//     let proposed_milestones1 = proposed_milestones.clone();
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         let _ = Currencies::deposit(CurrencyId::Native, &charlie, additional_amount);
//         create_project_multiple_milestones(alice, proposed_milestones);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let value = 500000u64;
//         Proposals::contribute(RuntimeOrigin::signed(bob), None, project_key, value).unwrap();
//
//         Proposals::contribute(RuntimeOrigin::signed(charlie), None, project_key, value).unwrap();
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(milestone1_key);
//         let _ = milestone_index.try_push(milestone2_key);
//
//         run_to_block(3);
//
//         Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();
//
//         Proposals::submit_milestone(RuntimeOrigin::signed(alice), project_key, milestone1_key)
//             .unwrap();
//
//         Proposals::submit_milestone(RuntimeOrigin::signed(alice), project_key, milestone2_key).ok();
//
//         run_to_block(5);
//         //Bob voting on the submitted milestone
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(bob),
//             project_key,
//             milestone1_key,
//             None,
//             true,
//         )
//         .ok();
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(bob),
//             project_key,
//             milestone2_key,
//             None,
//             true,
//         )
//         .ok();
//
//         //Charlie voting on the submitted milestone
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(charlie),
//             project_key,
//             milestone1_key,
//             None,
//             true,
//         )
//         .ok();
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(charlie),
//             project_key,
//             milestone2_key,
//             None,
//             true,
//         )
//         .ok();
//
//         assert_ok!(Proposals::finalise_milestone_voting(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             0
//         ));
//
//         assert_ok!(Proposals::finalise_milestone_voting(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             1
//         ));
//
//         assert_ok!(<proposals::Pallet<Test>>::withdraw(
//             RuntimeOrigin::signed(alice),
//             project_key
//         ));
//
//         //calculating the total percentage that can be withdrawn based on the submitted milestones
//         let initial_percentage_to_withdraw: u32 =
//             proposed_milestones1.get(0).unwrap().percentage_to_unlock
//                 + proposed_milestones1.get(1).unwrap().percentage_to_unlock;
//
//         //making sure that only balance is equal to the amount withdrawn
//         //making sure not all the required funds have been assigned instead only the percentage eligible could be withdrawn
//         assert_ne!(
//             Balances::free_balance(&alice),
//             additional_amount + required_funds
//         );
//         assert_eq!(
//             Balances::free_balance(&alice),
//             additional_amount + required_funds * (initial_percentage_to_withdraw as u64) / 100
//         );
//
//         // withdraw last milestone
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             2
//         ));
//         run_to_block(10);
//         //Bob voting on the submitted milestone
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(bob),
//             project_key,
//             milestone3_key,
//             None,
//             true,
//         )
//         .ok();
//         //Charlie voting on the submitted milestone
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(charlie),
//             project_key,
//             milestone3_key,
//             None,
//             true,
//         )
//         .ok();
//
//         assert_ok!(Proposals::finalise_milestone_voting(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             milestone3_key
//         ));
//
//         assert_ok!(<proposals::Pallet<Test>>::withdraw(
//             RuntimeOrigin::signed(alice),
//             project_key
//         ));
//
//         assert_eq!(
//             Balances::free_balance(&alice),
//             additional_amount + required_funds
//         );
//
//         //can withdraw only the amount corresponding to the milestone percentage completion
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::ProjectFundsWithdrawn(
//                 alice,
//                 0,
//                 500000u64,
//                 CurrencyId::Native
//             ))
//         );
//     })
// }
//
// #[test]
// fn test_project_initiator_can_withdraw_only_the_percentage_after_force_milestone_completed() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
//     let additional_amount = 10000000u64;
//     let required_funds = 1000000u64;
//
//     let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
//
//     let milestone1: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 1"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 20,
//     };
//     let milestone2: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 2"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 30,
//     };
//
//     let milestone3: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 3"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 50,
//     };
//     proposed_milestones.push(milestone1);
//     proposed_milestones.push(milestone2);
//     proposed_milestones.push(milestone3);
//     let proposed_milestones1 = proposed_milestones.clone();
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         let _ = Currencies::deposit(CurrencyId::Native, &charlie, additional_amount);
//         create_project_multiple_milestones(alice, proposed_milestones);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let value = 500000u64;
//         Proposals::contribute(RuntimeOrigin::signed(bob), None, project_key, value).unwrap();
//         Proposals::contribute(RuntimeOrigin::signed(charlie), None, project_key, value).unwrap();
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(0);
//         let _ = milestone_index.try_push(1);
//
//         run_to_block(3);
//
//         Proposals::approve(
//             RuntimeOrigin::root(),
//             None,
//             project_key,
//             Some(milestone_index),
//         )
//         .unwrap();
//
//         assert_ok!(<proposals::Pallet<Test>>::withdraw(
//             RuntimeOrigin::signed(alice),
//             project_key
//         ));
//
//         //calculating the total percentage that can be withdrawn based on the submitted milestones
//         let total_percentage_to_withdraw: u32 =
//             proposed_milestones1.get(0).unwrap().percentage_to_unlock
//                 + proposed_milestones1.get(1).unwrap().percentage_to_unlock;
//
//         //making sure that only balance is equal to the amount withdrawn
//         //making sure not all the required funds have been assigned instead only the percentage eligible could be withdrawn
//         assert_ne!(
//             Balances::free_balance(&alice),
//             additional_amount + required_funds
//         );
//         assert_eq!(
//             Balances::free_balance(&alice),
//             additional_amount + required_funds * (total_percentage_to_withdraw as u64) / 100
//         );
//
//         //can withdraw only the amount corresponding to the milestone percentage completion
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::ProjectFundsWithdrawn(
//                 alice,
//                 0,
//                 500000u64,
//                 CurrencyId::Native
//             ))
//         );
//     })
// }
//
// #[test]
// fn test_withdraw_upon_project_approval_and_finalised_voting() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//     let milestone1_key = 0;
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project(alice);
//
//         let project_key = 0;
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let required_funds = 100u64;
//         Proposals::contribute(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key,
//             required_funds,
//         )
//         .unwrap();
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(0);
//
//         run_to_block(3);
//
//         Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();
//
//         Proposals::submit_milestone(RuntimeOrigin::signed(alice), project_key, 0).unwrap();
//
//         run_to_block(5);
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(bob),
//             project_key,
//             milestone1_key,
//             None,
//             true,
//         )
//         .unwrap();
//
//         Proposals::finalise_milestone_voting(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             milestone1_key,
//         )
//         .unwrap();
//
//         assert_ok!(Proposals::withdraw(
//             RuntimeOrigin::signed(alice),
//             project_key
//         ));
//
//         assert_eq!(
//             Balances::free_balance(&alice),
//             additional_amount + required_funds
//         );
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::ProjectFundsWithdrawn(
//                 alice,
//                 0,
//                 100,
//                 CurrencyId::Native
//             ))
//         );
//     });
// }
//
// #[test]
// fn test_withdraw_from_non_initiator_account() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project(alice);
//
//         let project_key = 0;
//
//         assert_noop!(
//             Proposals::withdraw(RuntimeOrigin::signed(bob), project_key),
//             DispatchErrorWithPostInfo {
//                 post_info: PostDispatchInfo {
//                     actual_weight: None,
//                     pays_fee: Pays::Yes,
//                 },
//                 error: Error::<Test>::InvalidAccount.into(),
//             }
//         );
//     });
// }
//
// #[test]
// //positive test case submit multiple milestones
// fn submit_multiple_milestones() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let additional_amount = 100000000u64;
//     let voting_round1_key = 2;
//     let voting_round2_key = 3;
//     let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
//     let milestone1: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 1"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 50,
//     };
//     let milestone2: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 2"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 50,
//     };
//     proposed_milestones.push(milestone1);
//     proposed_milestones.push(milestone2);
//
//     let project_keys: BoundedProjectKeys = bounded_vec![0];
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project_multiple_milestones(alice, proposed_milestones);
//
//         let project_key = 0;
//         let milestone_index_1 = 0;
//         let milestone_index_2 = 1;
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let value = 100u64;
//         Proposals::contribute(RuntimeOrigin::signed(bob), None, project_key, value).unwrap();
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(milestone_index_1);
//         let _ = milestone_index.try_push(milestone_index_2);
//
//         run_to_block(3);
//
//         Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();
//
//         Proposals::submit_milestone(RuntimeOrigin::signed(alice), project_key, milestone_index_1)
//             .unwrap();
//
//         let voting_round_event_1 = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             voting_round_event_1,
//             mock::RuntimeEvent::from(proposals::Event::VotingRoundCreated(
//                 voting_round1_key,
//                 vec![project_key]
//             ))
//         );
//
//         run_to_block(5);
//
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             milestone_index_2
//         ));
//
//         let voting_round_event_2 = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             voting_round_event_2,
//             mock::RuntimeEvent::from(proposals::Event::VotingRoundCreated(
//                 voting_round2_key,
//                 vec![project_key]
//             ))
//         );
//     });
// }
//
// #[test]
// fn create_a_test_project_and_schedule_round_and_contribute_and_refund() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         //create_project extrinsic
//         create_project(alice);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         let project_key: u32 = 0;
//         let contribution_amount = 2000u64;
//
//         //schedule_round extrinsic
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number() + 1,
//             System::block_number() + 10,
//             //Project key starts with 0 for the first project submitted to the chain
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//         let additional_amount = 10_000;
//         let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);
//
//         run_to_block(4);
//         //contribute extrinsic
//         Proposals::contribute(
//             RuntimeOrigin::signed(alice),
//             None,
//             project_key,
//             contribution_amount,
//         )
//         .unwrap();
//
//         //ensuring alice's balance has reduced after contribution
//         let alice_balance_post_contribute: u64 = 8_000;
//         assert_eq!(
//             alice_balance_post_contribute,
//             Balances::free_balance(&alice)
//         );
//
//         Proposals::refund(RuntimeOrigin::root(), project_key).unwrap();
//
//         let exp_projectfundsrefunded_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             exp_projectfundsrefunded_event,
//             mock::RuntimeEvent::from(proposals::Event::ProjectFundsAddedToRefundQueue(
//                 project_key,
//                 contribution_amount
//             ))
//         );
//
//         // wait some blocks
//         run_to_block(System::block_number() + 1);
//
//         //ensuring the refunded amount was transferred back successfully
//         assert_eq!(additional_amount, Balances::free_balance(&alice));
//     });
// }
//
// #[test]
// fn withdraw_percentage_milestone_completed_refund_locked_milestone() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
//     let additional_amount = 10000000u64;
//     let required_funds = 1000000u64;
//     let project_key = 0;
//
//     let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
//
//     let milestone1: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 1"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 20,
//     };
//     let milestone2: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 2"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 30,
//     };
//
//     let milestone3: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 3"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 50,
//     };
//     proposed_milestones.push(milestone1);
//     proposed_milestones.push(milestone2);
//     proposed_milestones.push(milestone3);
//     let proposed_milestones1 = proposed_milestones.clone();
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         let _ = Currencies::deposit(CurrencyId::Native, &charlie, additional_amount);
//         create_project_multiple_milestones(alice, proposed_milestones);
//
//         let project_keys: BoundedProjectKeys = bounded_vec![0];
//         let milestone1_key = 0;
//         let milestone2_key = 1;
//
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 1,
//             project_keys,
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         let value = 500000u64;
//         Proposals::contribute(RuntimeOrigin::signed(bob), None, project_key, value).unwrap();
//         Proposals::contribute(RuntimeOrigin::signed(charlie), None, project_key, value).unwrap();
//
//         let mut milestone_index: BoundedMilestoneKeys = bounded_vec![];
//         let _ = milestone_index.try_push(0);
//
//         run_to_block(3);
//
//         Proposals::approve(
//             RuntimeOrigin::root(),
//             None,
//             project_key,
//             Some(milestone_index),
//         )
//         .unwrap();
//
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             milestone1_key
//         ));
//
//         assert_ok!(Proposals::submit_milestone(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             milestone2_key
//         ));
//
//         run_to_block(5);
//         //Bob voting on the submitted milestone
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(bob),
//             project_key,
//             milestone1_key,
//             None,
//             true,
//         )
//         .ok();
//
//         //Charlie voting on the submitted milestone
//         Proposals::vote_on_milestone(
//             RuntimeOrigin::signed(charlie),
//             project_key,
//             milestone1_key,
//             None,
//             true,
//         )
//         .ok();
//
//         assert_ok!(Proposals::finalise_milestone_voting(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             0
//         ));
//
//         assert_ok!(<proposals::Pallet<Test>>::withdraw(
//             RuntimeOrigin::signed(alice),
//             project_key
//         ));
//
//         //calculating the total percentage that can be withdrawn based on the submitted milestones
//         let total_percentage_to_withdraw: u32 =
//             proposed_milestones1.get(0).unwrap().percentage_to_unlock;
//
//         //making sure that only balance is equal to the amount withdrawn
//         //making sure not all the required funds have been assigned instead only the percentage eligible could be withdrawn
//         //checking that Alice now has 10.2m
//         assert_ne!(
//             Balances::free_balance(&alice),
//             additional_amount + required_funds
//         );
//         assert_eq!(
//             Balances::free_balance(&alice),
//             additional_amount + required_funds * (total_percentage_to_withdraw as u64) / 100
//         );
//
//         //can withdraw only the amount corresponding to the milestone percentage completion
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::ProjectFundsWithdrawn(
//                 alice,
//                 project_key,
//                 200000u64,
//                 CurrencyId::Native
//             ))
//         );
//
//         //validating contributor current balance
//         let contributor_balance_pre_refund: u64 = 9_500_000;
//         assert_eq!(contributor_balance_pre_refund, Balances::free_balance(&bob));
//         assert_eq!(
//             contributor_balance_pre_refund,
//             Balances::free_balance(&charlie)
//         );
//
//         Proposals::refund(RuntimeOrigin::root(), project_key).unwrap();
//
//         let exp_projectfundsrefunded_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             exp_projectfundsrefunded_event,
//             mock::RuntimeEvent::from(proposals::Event::ProjectFundsAddedToRefundQueue(
//                 project_key,
//                 800000u64
//             ))
//         );
//
//         // Wait a block so refunds occur in hook.
//         run_to_block(System::block_number() + 1);
//
//         //ensuring the refunded amount was transferred back successfully
//         let contributor_balance_pre_refund: u64 = 9_900_000;
//         assert_eq!(contributor_balance_pre_refund, Balances::free_balance(&bob));
//         assert_eq!(
//             contributor_balance_pre_refund,
//             Balances::free_balance(&charlie)
//         );
//     })
// }
//
// #[test]
// fn test_schedule_round_fails_gracefully_with_empty_vec() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     build_test_externality().execute_with(|| {
//         create_project(alice);
//
//         assert_noop!(
//             Proposals::schedule_round(
//                 RuntimeOrigin::root(),
//                 System::block_number(),
//                 System::block_number() + 1,
//                 // Empty keys is the test.
//                 bounded_vec![],
//                 RoundType::ContributionRound
//             ),
//             Error::<Test>::LengthMustExceedZero
//         );
//     });
// }
//
// #[test]
// fn test_raising_a_vote_of_no_confidence() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
//
//     let project_key = 0u32;
//
//     build_test_externality().execute_with(|| {
//         // Create a project for both alice and bob.
//         create_project(alice);
//
//         // Schedule a round to allow for contributions.
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         // Deposit funds and contribute.
//         let _ = Currencies::deposit(CurrencyId::Native, &bob, 10_000_000u64);
//         run_to_block(System::block_number() + 3);
//
//         Proposals::contribute(
//             RuntimeOrigin::signed(bob),
//             Some(1),
//             project_key,
//             1_000_000u64,
//         )
//         .unwrap();
//         run_to_block(System::block_number() + 101);
//
//         Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();
//
//         // Assert that Bob cannot raise the vote as he is not a contributor.
//         assert_noop!(
//             Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(charlie), project_key),
//             Error::<Test>::OnlyContributorsCanVote
//         );
//
//         // Call a vote of no confidence and assert it will pass.
//         assert_ok!(Proposals::raise_vote_of_no_confidence(
//             RuntimeOrigin::signed(bob),
//             project_key
//         ));
//
//         let vote = NoConfidenceVotes::<Test>::get(project_key).unwrap();
//         let round_count = RoundCount::<Test>::get();
//
//         // Assert that storage has been mutated correctly.
//         assert!(vote.nay == 1_000_000u64 && vote.yay == 0u64);
//         assert!(UserVotes::<Test>::get((bob, project_key, 0, round_count)) == Some(true));
//         assert!(round_count == 2u32);
//         assert!(NoConfidenceVotes::<Test>::contains_key(project_key));
//
//         // Assert that you cannot raise the vote twice.
//         assert_noop!(
//             Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(bob), project_key),
//             Error::<Test>::RoundStarted
//         );
//     });
// }
//
// #[test]
// fn test_adding_vote_of_no_confidence() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("charlie");
//     let project_key = 0u32;
//     build_test_externality().execute_with(|| {
//         // Create a project for both alice and bob.
//         create_project(alice);
//
//         //schedule a round to allow for contributions.
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         // Deposit funds and contribute.
//         let _ = Currencies::deposit(CurrencyId::Native, &charlie, 10_000_000u64);
//         let _ = Currencies::deposit(CurrencyId::Native, &bob, 20_000_000u64);
//         run_to_block(System::block_number() + 3);
//
//         // Setup required state to start voting: must have contributed and round must have started.
//         Proposals::contribute(
//             RuntimeOrigin::signed(charlie),
//             Some(1),
//             project_key,
//             500_000u64,
//         )
//         .unwrap();
//         Proposals::contribute(RuntimeOrigin::signed(bob), Some(1), project_key, 500_000u64)
//             .unwrap();
//
//         run_to_block(System::block_number() + 101);
//
//         // Assert that threshold has been met
//         assert_ok!(Proposals::approve(
//             RuntimeOrigin::root(),
//             Some(1),
//             project_key,
//             None
//         ));
//
//         assert_ok!(Proposals::raise_vote_of_no_confidence(
//             RuntimeOrigin::signed(charlie),
//             project_key
//         ));
//
//         // Charlie has raised a vote of no confidence, now Bob is gonna disagree!
//         assert_ok!(Proposals::vote_on_no_confidence_round(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key,
//             true
//         ));
//
//         // Assert Bob cannot game the system.
//         assert_noop!(
//             Proposals::vote_on_no_confidence_round(
//                 RuntimeOrigin::signed(bob),
//                 None,
//                 project_key,
//                 true
//             ),
//             Error::<Test>::VoteAlreadyExists
//         );
//         assert_noop!(
//             Proposals::vote_on_no_confidence_round(
//                 RuntimeOrigin::signed(bob),
//                 None,
//                 project_key,
//                 false
//             ),
//             Error::<Test>::VoteAlreadyExists
//         );
//
//         // Assert the state of the system is as it should be.
//         let vote = NoConfidenceVotes::<Test>::get(project_key).unwrap();
//         let round_count = RoundCount::<Test>::get();
//
//         // Assert that storage has been mutated correctly.
//         assert!(vote.nay == 500_000u64 && vote.yay == 500_000u64);
//         assert!(UserVotes::<Test>::get((charlie, project_key, 0, round_count)) == Some(true));
//         assert!(UserVotes::<Test>::get((bob, project_key, 0, round_count)) == Some(true));
//
//         assert!(round_count == 2u32);
//     });
// }
//
// #[test]
// fn test_finalise_vote_of_no_confidence_with_threshold_met() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("charlie");
//     let steve = get_account_id_from_seed::<sr25519::Public>("steve");
//
//     let project_key = 0u32;
//     build_test_externality().execute_with(|| {
//         // Create a project for both alice and bob.
//         create_project(alice);
//
//         //schedule a round to allow for contributions.
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         // Deposit funds and contribute.
//         let _ = Currencies::deposit(CurrencyId::Native, &charlie, 10_000_000u64);
//         let _ = Currencies::deposit(CurrencyId::Native, &bob, 20_000_000u64);
//         run_to_block(System::block_number() + 3);
//         // Setup required state to start voting: must have contributed and round must have started.
//         Proposals::contribute(
//             RuntimeOrigin::signed(charlie),
//             Some(1),
//             project_key,
//             750_001u64,
//         )
//         .unwrap();
//         Proposals::contribute(RuntimeOrigin::signed(bob), Some(1), project_key, 250_000u64)
//             .unwrap();
//         run_to_block(System::block_number() + 101);
//         Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();
//
//         assert_ok!(Proposals::raise_vote_of_no_confidence(
//             RuntimeOrigin::signed(charlie),
//             project_key
//         ));
//         assert_ok!(Proposals::vote_on_no_confidence_round(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key,
//             false
//         ));
//
//         // Assert that steve who is not a contributor cannot finalise the same goes for the initiator.
//         assert_noop!(
//             Proposals::finalise_no_confidence_round(
//                 RuntimeOrigin::signed(steve),
//                 None,
//                 project_key
//             ),
//             Error::<Test>::OnlyContributorsCanVote
//         );
//         assert_noop!(
//             Proposals::finalise_no_confidence_round(
//                 RuntimeOrigin::signed(alice),
//                 None,
//                 project_key
//             ),
//             Error::<Test>::OnlyContributorsCanVote
//         );
//         // And we might aswell assert that you cannot call finalise on a project key that doesnt exist.
//         assert_noop!(
//             Proposals::finalise_no_confidence_round(RuntimeOrigin::signed(bob), None, 2),
//             Error::<Test>::ProjectNotInRound
//         );
//         // Assert that bob, a contrbutor, can finalise
//         assert_ok!(Proposals::finalise_no_confidence_round(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key
//         ));
//     });
// }
//
// // I Realised that i have already tested for thresholds on the mark and therefore above
// // Alas i should test below the threshold
// #[test]
// fn test_finalise_vote_of_no_confidence_below_threshold() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("charlie");
//
//     let project_key = 0u32;
//     build_test_externality().execute_with(|| {
//         // Create a project for both alice and bob.
//         create_project(alice);
//
//         //schedule a round to allow for contributions.
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         // Deposit funds and contribute.
//         let _ = Currencies::deposit(CurrencyId::Native, &charlie, 10_000_000u64);
//         let _ = Currencies::deposit(CurrencyId::Native, &bob, 20_000_000u64);
//         run_to_block(System::block_number() + 3);
//
//         // Setup required state to start voting: must have contributed and round must have started.
//         Proposals::contribute(
//             RuntimeOrigin::signed(charlie),
//             Some(1),
//             project_key,
//             500_000u64,
//         )
//         .unwrap();
//         Proposals::contribute(RuntimeOrigin::signed(bob), Some(1), project_key, 500_000u64)
//             .unwrap();
//
//         run_to_block(System::block_number() + 101);
//
//         // Assert that threshold has been met
//         assert_ok!(Proposals::approve(
//             RuntimeOrigin::root(),
//             Some(1),
//             project_key,
//             None
//         ));
//
//         assert_ok!(Proposals::raise_vote_of_no_confidence(
//             RuntimeOrigin::signed(charlie),
//             project_key
//         ));
//         assert_ok!(Proposals::vote_on_no_confidence_round(
//             RuntimeOrigin::signed(bob),
//             Some(2),
//             project_key,
//             true
//         ));
//
//         assert_noop!(
//             Proposals::finalise_no_confidence_round(
//                 RuntimeOrigin::signed(charlie),
//                 Some(2),
//                 project_key
//             ),
//             Error::<Test>::VoteThresholdNotMet
//         );
//     });
// }
//
// #[test]
// fn test_finalise_vote_of_no_confidence_refunds_contributors() {
//     // The project creator.
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     // The contributors.
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let charlie = get_account_id_from_seed::<sr25519::Public>("charlie");
//
//     let project_key = 0u32;
//     build_test_externality().execute_with(|| {
//         // Create a project for both alice and bob.
//         create_project(alice);
//         let _ = Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//         let _ = Currencies::deposit(CurrencyId::Native, &charlie, 1_000_000u64);
//         let _ = Currencies::deposit(CurrencyId::Native, &bob, 1_000_000u64);
//         run_to_block(System::block_number() + 3);
//         let _ = Proposals::contribute(
//             RuntimeOrigin::signed(charlie),
//             Some(1),
//             project_key,
//             750_000u64,
//         )
//         .unwrap();
//         let _ = Proposals::contribute(RuntimeOrigin::signed(bob), Some(1), project_key, 250_000u64)
//             .unwrap();
//         run_to_block(System::block_number() + 101);
//
//         // assert that the voters have had their funds transferred.
//         assert!(Currencies::free_balance(CurrencyId::Native, &charlie) == 250_000u64);
//         assert!(Currencies::free_balance(CurrencyId::Native, &bob) == 750_000u64);
//
//         // approve and raise votees
//         let _ = Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();
//         let _ = Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(charlie), project_key)
//             .unwrap();
//         let _ = Proposals::vote_on_no_confidence_round(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key,
//             false,
//         )
//         .unwrap();
//
//         // Assert that bob, a contrbutor, can finalise
//         assert_ok!(Proposals::finalise_no_confidence_round(
//             RuntimeOrigin::signed(bob),
//             None,
//             project_key
//         ));
//
//         // Wait a block so that refunds occur;
//         run_to_block(System::block_number() + 1);
//         // assert that the voters have had their funds refunded.
//         assert!(Currencies::free_balance(CurrencyId::Native, &charlie) == 1_000_000u64);
//         assert!(Currencies::free_balance(CurrencyId::Native, &bob) == 1_000_000u64);
//     });
// }
//
// // Very slow test, due to the creation of multiple account keys.
// #[test]
// fn test_refunds_go_back_to_contributors() {
//     build_test_externality().execute_with(|| {
//         let initiator = get_account_id_from_seed::<sr25519::Public>("TreasuryPot");
//         let mut accounts: Vec<<Test as frame_system::Config>::AccountId> = vec![];
//         let num_of_refunds: u32 = 100;
//
//         create_project(initiator);
//         let _ = Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![0u32],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         run_to_block(System::block_number() + 2u64);
//         let input: Vec<String> = (0..num_of_refunds).map(|i| i.to_string()).collect();
//         for i in 0..num_of_refunds {
//             let acc = get_account_id_from_seed::<sr25519::Public>(&input[i as usize].as_str());
//             accounts.push(acc.clone());
//             let _ = Currencies::deposit(CurrencyId::Native, &acc.clone(), 20_000u64);
//             let _ = Proposals::contribute(RuntimeOrigin::signed(acc), Some(1), 0u32, 10_000u64)
//                 .unwrap();
//         }
//
//         assert_ok!(Proposals::refund(RuntimeOrigin::root(), 0));
//
//         // The maximum amount of block it should take for all refunds to occur.
//         run_to_block(num_of_refunds as u64 / RefundsPerBlock::get() as u64);
//
//         for i in 0..num_of_refunds {
//             assert_eq!(
//                 Currencies::free_balance(CurrencyId::Native, &accounts[i as usize]),
//                 20_000u64
//             );
//         }
//
//         assert!(
//             Currencies::free_balance(CurrencyId::Native, &Proposals::project_account_id(0)) == 0u64
//         )
//     });
// }
//
// #[test]
// fn test_refunds_state_is_handled_correctly() {
//     build_test_externality().execute_with(|| {
//         let initiator = get_account_id_from_seed::<sr25519::Public>("TreasuryPot");
//         let mut accounts: Vec<<Test as frame_system::Config>::AccountId> = vec![];
//         // Only works if
//         let num_of_refunds: u32 = 20;
//
//         create_project(initiator);
//         let _ = Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![0u32],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         run_to_block(System::block_number() + 2u64);
//         let input: Vec<String> = (0..num_of_refunds).map(|i| i.to_string()).collect();
//         for i in 0..num_of_refunds {
//             let acc = get_account_id_from_seed::<sr25519::Public>(&input[i as usize].as_str());
//             accounts.push(acc.clone());
//             let _ = Currencies::deposit(CurrencyId::Native, &acc.clone(), 20_000u64);
//             let _ = Proposals::contribute(RuntimeOrigin::signed(acc), Some(1), 0u32, 10_000u64)
//                 .unwrap();
//         }
//
//         assert_ok!(Proposals::refund(RuntimeOrigin::root(), 0));
//         let mut refunds_completed = 0usize;
//         // The maximum amount of block it should take for all refunds to occur.
//         for _ in 0..(num_of_refunds / RefundsPerBlock::get() as u32) {
//             run_to_block_with_no_idle_space(System::block_number() + 1u64);
//
//             // Get the total number of refunds completed.
//             let refunds_after_block = accounts
//                 .iter()
//                 .map(|acc| Currencies::free_balance(CurrencyId::Native, acc))
//                 .filter(|balance| balance == &20_000u64)
//                 .collect::<Vec<u64>>()
//                 .len();
//
//             // Assert that only 2 have been completed
//             assert_eq!(refunds_after_block - refunds_completed, 2usize);
//             refunds_completed += 2;
//
//             // And that they have been removed from the refund list.
//             assert_eq!(
//                 RefundQueue::<Test>::get().len(),
//                 num_of_refunds as usize - refunds_completed
//             );
//         }
//
//         assert!(
//             Currencies::free_balance(CurrencyId::Native, &Proposals::project_account_id(0)) == 0u64
//         )
//     });
// }
//
// // create project, schedule a round, approve and submit a milestone.
// // assert that the vote will pass when it is on the threshold.
// #[test]
// fn test_finalise_milestone_is_ok_on_threshold_vote() {
//     build_test_externality().execute_with(|| {
//         let initiator = get_account_id_from_seed::<sr25519::Public>("initiator");
//         let contyes = get_account_id_from_seed::<sr25519::Public>("cont1");
//         let contno = get_account_id_from_seed::<sr25519::Public>("cont2");
//
//         create_project(initiator);
//         let _ = Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![0u32],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//
//         // Deposit and contribute up to the voting threshold so that it should pass.
//         let _ = Currencies::deposit(CurrencyId::Native, &contyes, 1_000_000u64);
//         let _ = Currencies::deposit(CurrencyId::Native, &contno, 1_000_000u64);
//
//         let yes_contribution = 1_000_000u64 / 100u64 * PercentRequiredForVoteToPass::get() as u64;
//         let no_contribution =
//             1_000_000u64 / 100u64 * (100u8 - PercentRequiredForVoteToPass::get()) as u64;
//
//         run_to_block(System::block_number() + 1);
//
//         let _ = Proposals::contribute(
//             RuntimeOrigin::signed(contyes.clone()),
//             Some(1),
//             0u32,
//             yes_contribution,
//         )
//         .unwrap();
//         let _ = Proposals::contribute(
//             RuntimeOrigin::signed(contno.clone()),
//             Some(1),
//             0u32,
//             no_contribution,
//         )
//         .unwrap();
//
//         run_to_block(System::block_number() + 100);
//
//         // Assert that threshold has been met
//         let _ = Proposals::approve(RuntimeOrigin::root(), Some(1), 0, None).unwrap();
//
//         let _ =
//             Proposals::submit_milestone(RuntimeOrigin::signed(initiator.clone()), 0, 0).unwrap();
//
//         run_to_block(System::block_number() + 1);
//
//         let _ =
//             Proposals::vote_on_milestone(RuntimeOrigin::signed(contyes.clone()), 0, 0, None, true)
//                 .unwrap();
//         let _ =
//             Proposals::vote_on_milestone(RuntimeOrigin::signed(contno.clone()), 0, 0, None, false)
//                 .unwrap();
//
//         assert_ok!(Proposals::finalise_milestone_voting(
//             RuntimeOrigin::signed(initiator),
//             0,
//             0
//         ));
//     })
// }
//
// #[test]
// //update project required funds and milestones - positive test case
// fn update_an_existing_project() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//     let updated_project_name = b"Farmer's Project Sudan2"
//         .to_vec()
//         .try_into()
//         .expect("Invalid input");
//     let expected_project_name_in_event = b"Farmer's Project Sudan2"
//         .to_vec()
//         .try_into()
//         .expect("Invalid input");
//     let updated_project_logo = b"Some logo".to_vec().try_into().expect("Invalid input");
//     let updated_project_description = b"Raise funds for Farmer's project phase 2"
//         .to_vec()
//         .try_into()
//         .expect("Invalid input");
//     let updated_project_website = b"www.ab.com".to_vec().try_into().expect("Invalid input");
//     let additional_amount = 100000000u64;
//     let updated_required_funds = 2_500_000u64;
//     let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
//     let milestone1: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 1"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 20,
//     };
//     let milestone2: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 2"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 30,
//     };
//
//     let milestone3: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 3"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 50,
//     };
//     proposed_milestones.push(milestone1);
//     proposed_milestones.push(milestone2);
//     proposed_milestones.push(milestone3);
//
//     let mut updated_proposed_milestones: Vec<ProposedMilestone> = Vec::new();
//     let updated_milestone1: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 1"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 70,
//     };
//     let updated_milestone2: ProposedMilestone = ProposedMilestone {
//         name: b"milestone 2"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         percentage_to_unlock: 30,
//     };
//
//     updated_proposed_milestones.push(updated_milestone1);
//     updated_proposed_milestones.push(updated_milestone2);
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, additional_amount);
//         create_project_multiple_milestones(alice, proposed_milestones);
//
//         let project_key = 0;
//
//         assert_ok!(Proposals::update_project(
//             RuntimeOrigin::signed(alice),
//             project_key,
//             updated_project_name,
//             updated_project_logo,
//             updated_project_description,
//             updated_project_website,
//             updated_proposed_milestones
//                 .try_into()
//                 .expect("Invalid proposed milestones"),
//             updated_required_funds,
//             CurrencyId::Native,
//         ));
//
//         let latest_event = <frame_system::Pallet<Test>>::events()
//             .pop()
//             .expect("Expected at least one RuntimeEventRecord to be found")
//             .event;
//         assert_eq!(
//             latest_event,
//             mock::RuntimeEvent::from(proposals::Event::ProjectUpdated(
//                 alice,
//                 expected_project_name_in_event,
//                 project_key,
//                 updated_required_funds
//             ))
//         );
//     });
// }
//
// #[test]
// fn only_the_initiator_can_update_project() {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
//
//     build_test_externality().execute_with(|| {
//         deposit_initial_balance(&alice, &bob, 1000000);
//         create_project(alice.clone());
//         let updated_milestone1: ProposedMilestone = ProposedMilestone {
//             name: b"milestone 1"
//                 .to_vec()
//                 .try_into()
//                 .expect("input should be of decent length"),
//             percentage_to_unlock: 70,
//         };
//
//         assert_noop!(
//             Proposals::update_project(
//                 RuntimeOrigin::signed(bob),
//                 100000u32,
//                 b"abc".to_vec().try_into().expect("qed"),
//                 b"abc".to_vec().try_into().expect("qed"),
//                 b"abc".to_vec().try_into().expect("qed"),
//                 b"abc".to_vec().try_into().expect("qed"),
//                 vec![updated_milestone1].try_into().expect("qed"),
//                 10000,
//                 CurrencyId::Native,
//             ),
//             Error::<Test>::InvalidAccount
//         );
//     })
// }
//
// //common helper methods
// fn create_project(account: AccountId) {
//     assert_ok!(Proposals::create_project(
//         RuntimeOrigin::signed(account),
//         //project name
//         b"Farmer's Project Sudan"
//             .to_vec()
//             .try_into()
//             .expect("test bytes should be of decent length;"),
//         //project logo
//         b"Imbue Logo"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //project description
//         b"This project is aimed at providing decentralised funding for a farming project."
//             .to_vec()
//             .try_into()
//             .expect("test bytes should be of decent length;"),
//         //website
//         b"https://farmers.network"
//             .to_vec()
//             .try_into()
//             .expect("test bytes should be of decent length;"),
//         //milestone
//         bounded_vec![ProposedMilestone {
//             name: bounded_vec![],
//             percentage_to_unlock: 100
//         }],
//         //funds required
//         1_000_000u64,
//         CurrencyId::Native
//     ));
// }
//
// fn create_project_multiple_milestones(
//     alice: AccountId,
//     proposed_milestones: Vec<ProposedMilestone>,
// ) {
//     assert_ok!(Proposals::create_project(
//         RuntimeOrigin::signed(alice),
//         //project name
//         b"Farmer's Project Sudan"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //project logo
//         b"Imbue Logo"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //project description
//         b"This project is aimed at providing decentralised funding for a farming project."
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //website
//         b"https://farmers.network"
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //milestone
//         proposed_milestones
//             .try_into()
//             .expect("proposed milestones are too long"),
//         //funds required
//         1_000_000u64,
//         CurrencyId::Native
//     ));
// }
//
// fn create_projects_with_inputs(
//     name: &str,
//     logo: &str,
//     description: &str,
//     website: &str,
//     funds_required: u64,
// ) -> DispatchResultWithPostInfo {
//     let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
//     Proposals::create_project(
//         RuntimeOrigin::signed(alice),
//         //project name
//         name.as_bytes()
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //project logo
//         logo.as_bytes()
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //project description
//         description
//             .as_bytes()
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //website
//         website
//             .as_bytes()
//             .to_vec()
//             .try_into()
//             .expect("input should be of decent length"),
//         //milestone
//         bounded_vec![ProposedMilestone {
//             name: bounded_vec![],
//             percentage_to_unlock: 100
//         }],
//         //funds required
//         funds_required,
//         CurrencyId::Native,
//     )
// }
//
// fn deposit_initial_balance(alice: &AccountId, bob: &AccountId, additional_amount: u64) {
//     let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);
//     let _ = Currencies::deposit(CurrencyId::Native, &bob, additional_amount);
// }
//
// fn run_to_block(n: u64) {
//     while System::block_number() < n {
//         System::set_block_number(System::block_number() + 1);
//         System::on_initialize(System::block_number());
//         Proposals::on_initialize(System::block_number());
//         //Bad case scenario is that we have little space. all tests must still pass.
//         if n % 2 == 0 {
//             Proposals::on_idle(System::block_number(), Weight::MAX / 90);
//         } else {
//             Proposals::on_idle(System::block_number(), Weight::MAX / 2);
//         }
//     }
// }
//
// fn run_to_block_with_no_idle_space(n: u64) {
//     while System::block_number() < n {
//         System::set_block_number(System::block_number() + 1);
//         System::on_initialize(System::block_number());
//         Proposals::on_initialize(System::block_number());
//         Proposals::on_idle(System::block_number(), Weight::zero());
//     }
// }
