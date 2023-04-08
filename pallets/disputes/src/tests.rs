use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};



// #[test]
// fn test_raising_a_vote_of_no_confidence() {
//     let project_key = 0u32;

//     build_test_externality().execute_with(|| {
//         // Create a project for both ALICE and BOB.
//         assert_ok!(create_project());

//         // Schedule a round to allow for contributions.
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();

//         // Deposit funds and contribute.
//         run_to_block(System::block_number() + 3);

//         Proposals::contribute(
//             RuntimeOrigin::signed(*BOB),
//             Some(1),
//             project_key,
//             1_000_000u64,
//         )
//         .unwrap();
//         run_to_block(System::block_number() + 101);

//         Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();

//         // Assert that Bob cannot raise the vote as he is not a contributor.
//         assert_noop!(
//             Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*CHARLIE), project_key),
//             Error::<Test>::OnlyContributorsCanVote
//         );

//         // Call a vote of no confidence and assert it will pass.
//         assert_ok!(Proposals::raise_vote_of_no_confidence(
//             RuntimeOrigin::signed(*BOB),
//             project_key
//         ));

//         let vote = NoConfidenceVotes::<Test>::get(project_key).unwrap();
//         let round_count = RoundCount::<Test>::get();

//         // Assert that storage has been mutated correctly.
//         assert!(vote.nay == 1_000_000u64 && vote.yay == 0u64);
//         assert!(UserVotes::<Test>::get((*BOB, project_key, 0, round_count)) == Some(true));
//         assert!(round_count == 2u32);
//         assert!(NoConfidenceVotes::<Test>::contains_key(project_key));

//         // Assert that you cannot raise the vote twice.
//         assert_noop!(
//             Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*BOB), project_key),
//             Error::<Test>::RoundStarted
//         );
//     });
// }

// #[test]
// fn test_adding_vote_of_no_confidence() {
//     let project_key = 0u32;
//     build_test_externality().execute_with(|| {
//         // Create a project for both ALICE and BOB.
//         assert_ok!(create_project());

//         //schedule a round to allow for contributions.
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();

//         // Deposit funds and contribute.
//         run_to_block(System::block_number() + 3);

//         // Setup required state to start voting: must have contributed and round must have started.
//         Proposals::contribute(
//             RuntimeOrigin::signed(*CHARLIE),
//             Some(1),
//             project_key,
//             500_000u64,
//         )
//         .unwrap();
//         Proposals::contribute(
//             RuntimeOrigin::signed(*BOB),
//             Some(1),
//             project_key,
//             500_000u64,
//         )
//         .unwrap();

//         run_to_block(System::block_number() + 101);

//         // Assert that threshold has been met
//         assert_ok!(Proposals::approve(
//             RuntimeOrigin::root(),
//             Some(1),
//             project_key,
//             None
//         ));

//         assert_ok!(Proposals::raise_vote_of_no_confidence(
//             RuntimeOrigin::signed(*CHARLIE),
//             project_key
//         ));

//         // Charlie has raised a vote of no confidence, now Bob is gonna disagree!
//         assert_ok!(Proposals::vote_on_no_confidence_round(
//             RuntimeOrigin::signed(*BOB),
//             None,
//             project_key,
//             true
//         ));

//         // Assert Bob cannot game the system.
//         assert_noop!(
//             Proposals::vote_on_no_confidence_round(
//                 RuntimeOrigin::signed(*BOB),
//                 None,
//                 project_key,
//                 true
//             ),
//             Error::<Test>::VoteAlreadyExists
//         );
//         assert_noop!(
//             Proposals::vote_on_no_confidence_round(
//                 RuntimeOrigin::signed(*BOB),
//                 None,
//                 project_key,
//                 false
//             ),
//             Error::<Test>::VoteAlreadyExists
//         );

//         // Assert the state of the system is as it should be.
//         let vote = NoConfidenceVotes::<Test>::get(project_key).unwrap();
//         let round_count = RoundCount::<Test>::get();

//         // Assert that storage has been mutated correctly.
//         assert!(vote.nay == 500_000u64 && vote.yay == 500_000u64);
//         assert!(UserVotes::<Test>::get((*CHARLIE, project_key, 0, round_count)) == Some(true));
//         assert!(UserVotes::<Test>::get((*BOB, project_key, 0, round_count)) == Some(true));

//         assert!(round_count == 2u32);
//     });
// }

// #[test]
// fn test_finalise_vote_of_no_confidence_with_threshold_met() {
//     let project_key = 0u32;
//     build_test_externality().execute_with(|| {
//         // Create a project for both ALICE and BOB.
//         assert_ok!(create_project());

//         //schedule a round to allow for contributions.
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();

//         // Deposit funds and contribute.
//         run_to_block(System::block_number() + 3);
//         // Setup required state to start voting: must have contributed and round must have started.
//         Proposals::contribute(
//             RuntimeOrigin::signed(*CHARLIE),
//             Some(1),
//             project_key,
//             750_001u64,
//         )
//         .unwrap();
//         Proposals::contribute(
//             RuntimeOrigin::signed(*BOB),
//             Some(1),
//             project_key,
//             250_000u64,
//         )
//         .unwrap();
//         run_to_block(System::block_number() + 101);
//         Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();

//         assert_ok!(Proposals::raise_vote_of_no_confidence(
//             RuntimeOrigin::signed(*CHARLIE),
//             project_key
//         ));
//         assert_ok!(Proposals::vote_on_no_confidence_round(
//             RuntimeOrigin::signed(*BOB),
//             None,
//             project_key,
//             false
//         ));

//         // Assert that steve who is not a contributor cannot finalise the same goes for the initiator.
//         assert_noop!(
//             Proposals::finalise_no_confidence_round(
//                 RuntimeOrigin::signed(*ALICE),
//                 None,
//                 project_key
//             ),
//             Error::<Test>::OnlyContributorsCanVote
//         );
//         assert_noop!(
//             Proposals::finalise_no_confidence_round(
//                 RuntimeOrigin::signed(*ALICE),
//                 None,
//                 project_key
//             ),
//             Error::<Test>::OnlyContributorsCanVote
//         );
//         // And we might aswell assert that you cannot call finalise on a project key that doesnt exist.
//         assert_noop!(
//             Proposals::finalise_no_confidence_round(RuntimeOrigin::signed(*BOB), None, 2),
//             Error::<Test>::ProjectNotInRound
//         );
//         // Assert that BOB, a contrbutor, can finalise
//         assert_ok!(Proposals::finalise_no_confidence_round(
//             RuntimeOrigin::signed(*BOB),
//             None,
//             project_key
//         ));
//     });
// }

// // I Realised that i have already tested for thresholds on the mark and therefore above
// // Alas i should test below the threshold
// #[test]
// fn test_finalise_vote_of_no_confidence_below_threshold() {
//     let project_key = 0u32;
//     build_test_externality().execute_with(|| {
//         // Create a project for both ALICE and BOB.
//         assert_ok!(create_project());

//         //schedule a round to allow for contributions.
//         Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();

//         // Deposit funds and contribute.
//         run_to_block(System::block_number() + 3);

//         // Setup required state to start voting: must have contributed and round must have started.
//         Proposals::contribute(
//             RuntimeOrigin::signed(*CHARLIE),
//             Some(1),
//             project_key,
//             500_000u64,
//         )
//         .unwrap();
//         Proposals::contribute(
//             RuntimeOrigin::signed(*BOB),
//             Some(1),
//             project_key,
//             500_000u64,
//         )
//         .unwrap();

//         run_to_block(System::block_number() + 101);

//         // Assert that threshold has been met
//         assert_ok!(Proposals::approve(
//             RuntimeOrigin::root(),
//             Some(1),
//             project_key,
//             None
//         ));

//         assert_ok!(Proposals::raise_vote_of_no_confidence(
//             RuntimeOrigin::signed(*CHARLIE),
//             project_key
//         ));
//         assert_ok!(Proposals::vote_on_no_confidence_round(
//             RuntimeOrigin::signed(*BOB),
//             Some(2),
//             project_key,
//             true
//         ));

//         assert_noop!(
//             Proposals::finalise_no_confidence_round(
//                 RuntimeOrigin::signed(*CHARLIE),
//                 Some(2),
//                 project_key
//             ),
//             Error::<Test>::VoteThresholdNotMet
//         );
//     });
// }

// #[test]
// fn test_finalise_vote_of_no_confidence_refunds_contributors() {
//     // The project creator.

//     // The contributors.

//     build_test_externality().execute_with(|| {
//         let initial_balance = Tokens::free_balance(CurrencyId::Native, &*BOB);
//         let project_key = 0u32;
//         // Create a project for both ALICE and BOB.
//         assert_ok!(create_project());

//         let _ = Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![project_key],
//             RoundType::ContributionRound,
//         )
//         .unwrap();
//         run_to_block(System::block_number() + 3);
//         let _ = Proposals::contribute(
//             RuntimeOrigin::signed(*CHARLIE),
//             Some(1),
//             project_key,
//             750_000u64,
//         )
//         .unwrap();
//         let _ = Proposals::contribute(
//             RuntimeOrigin::signed(*BOB),
//             Some(1),
//             project_key,
//             250_000u64,
//         )
//         .unwrap();
//         run_to_block(System::block_number() + 101);

//         // assert that the voters have had their funds transferred.
//         assert_eq!(
//             Tokens::free_balance(CurrencyId::Native, &BOB),
//             initial_balance - 250_000u64
//         );
//         assert_eq!(
//             Tokens::free_balance(CurrencyId::Native, &CHARLIE),
//             initial_balance - 750_000
//         );

//         // approve and raise votees
//         let _ = Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();
//         let _ =
//             Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*CHARLIE), project_key)
//                 .unwrap();
//         let _ = Proposals::vote_on_no_confidence_round(
//             RuntimeOrigin::signed(*BOB),
//             None,
//             project_key,
//             false,
//         )
//         .unwrap();

//         // Assert that BOB, a contrbutor, can finalise
//         assert_ok!(Proposals::finalise_no_confidence_round(
//             RuntimeOrigin::signed(*BOB),
//             None,
//             project_key
//         ));

//         // Wait a block so that refunds occur;
//         run_to_block(System::block_number() + 1);
//         // assert that the voters have had their funds refunded.
//         assert_eq!(
//             Tokens::free_balance(CurrencyId::Native, &CHARLIE),
//             initial_balance
//         );
//         assert_eq!(
//             Tokens::free_balance(CurrencyId::Native, &BOB),
//             initial_balance
//         );
//     });
// }

// // Very slow test, due to the creation of multiple account keys.
// #[test]
// fn test_refunds_go_back_to_contributors() {
//     build_test_externality().execute_with(|| {
//         let mut accounts: Vec<<Test as frame_system::Config>::AccountId> = vec![];
//         let num_of_refunds: u32 = 100;
//         assert_ok!(create_project());

//         let _ = Proposals::schedule_round(
//             RuntimeOrigin::root(),
//             System::block_number(),
//             System::block_number() + 100,
//             bounded_vec![0u32],
//             RoundType::ContributionRound,
//         )
//         .unwrap();

//         run_to_block(System::block_number() + 2u64);
//         let input: Vec<String> = (0..num_of_refunds).map(|i| i.to_string()).collect();
//         for i in 0..num_of_refunds {
//             let acc = get_account_id_from_seed::<sr25519::Public>(&input[i as usize].as_str());
//             accounts.push(acc.clone());
//             let _ = Tokens::deposit(CurrencyId::Native, &acc.clone(), 20_000u64);
//             let _ = Proposals::contribute(RuntimeOrigin::signed(acc), Some(1), 0u32, 10_000u64)
//                 .unwrap();
//         }

//         assert_ok!(Proposals::refund(RuntimeOrigin::root(), 0));

//         // The maximum amount of block it should take for all refunds to occur.
//         run_to_block(num_of_refunds as u64 / RefundsPerBlock::get() as u64);

//         for i in 0..num_of_refunds {
//             assert_eq!(
//                 Tokens::free_balance(CurrencyId::Native, &accounts[i as usize]),
//                 20_000u64
//             );
//         }

//         assert!(
//             Currencies::free_balance(CurrencyId::Native, &Proposals::project_account_id(0)) == 0u64
//         )
//     });
// }
