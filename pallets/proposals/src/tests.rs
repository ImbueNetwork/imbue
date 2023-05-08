use crate as proposals;
use crate::mock::*;

use crate::*;
use common_types::CurrencyId;
use frame_support::{
    assert_noop, assert_ok, bounded_btree_map, bounded_vec,
    dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo},
};

use sp_core::H256;

use sp_std::vec::Vec;

#[test]
fn create_a_test_project() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());
    });
}

#[test]
fn create_a_test_project_with_less_than_100_percent() {
    build_test_externality().execute_with(|| {
        assert_noop!(
            Proposals::create_project(
                RuntimeOrigin::signed(*ALICE),
                gen_hash(1),
                bounded_vec![ProposedMilestone {
                    percentage_to_unlock: 99
                }],
                //funds required
                1000000u64,
                CurrencyId::Native
            ),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::MilestonesTotalPercentageMustEqual100.into()
            }
        );
    });
}

#[test]
fn create_a_test_project_and_add_whitelist() {
    let max_cap = 1_000_000u64;
    let project_key = 0;
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());
        let whitelist = bounded_btree_map!(*ALICE => max_cap);

        Proposals::add_project_whitelist(RuntimeOrigin::signed(*ALICE), project_key, whitelist)
            .unwrap();

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::WhitelistAdded(0, 1))
        );
    });
}

#[test]
fn create_a_test_project_and_add_whitelist_from_non_initiator_fail() {
    let max_cap = 1000000u64;
    let project_key = 0;
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let whitelist = bounded_btree_map!(*ALICE => max_cap);

        assert_noop!(
            Proposals::add_project_whitelist(RuntimeOrigin::signed(*BOB), project_key, whitelist),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::UserIsNotInitiator.into()
            }
        );
    });
}

#[test]
fn create_a_test_project_remove_whitelist() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        Proposals::remove_project_whitelist(RuntimeOrigin::signed(*ALICE), 0).unwrap();
        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::WhitelistRemoved(0, 1))
        );
    });
}

#[test]
fn create_a_test_project_and_schedule_round() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        assert_ok!(Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            //Project key starts with 0 for the first project submitted to the chain
            bounded_vec![0],
            RoundType::ContributionRound,
        ));
    });
}

#[test]
fn schedule_round_invalid_project_key() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        assert_noop!(
            Proposals::schedule_round(
                RuntimeOrigin::root(),
                System::block_number(),
                System::block_number() + 1,
                //Project key starts with 0 for the first project submitted to the chain
                bounded_vec![1],
                RoundType::ContributionRound
            ),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::ProjectDoesNotExist.into()
            }
        );
    });
}

#[test]
fn schedule_round_invalid_end_block_no() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        assert_noop!(
            Proposals::schedule_round(
                RuntimeOrigin::root(),
                System::block_number() + 6000,
                System::block_number() + 3000,
                //Project key starts with 0 for the first project submitted to the chain
                bounded_vec![1],
                RoundType::ContributionRound
            ),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::EndTooEarly.into()
            }
        );
    });
}

#[test]
fn cancel_round_no_active_round() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let _ = Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 3000,
            System::block_number() + 6000,
            bounded_vec![0],
            RoundType::ContributionRound,
        );

        assert_noop!(
            Proposals::cancel_round(RuntimeOrigin::root(), 0),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::NoActiveRound.into()
            }
        );
    });
}

#[test]
fn test_funding_round_is_created_on_schedule_round() {
    let project_keys: BoundedProjectKeys = bounded_vec![0u32];

    //create_project extrinsic
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 2,
            project_keys.clone(),
            RoundType::ContributionRound,
        )
        .unwrap();

        let exp_fundingroundcreated_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;

        assert_eq!(
            exp_fundingroundcreated_event,
            mock::RuntimeEvent::from(proposals::Event::FundingRoundCreated(
                1,
                project_keys.to_vec()
            ))
        );
    });
}

#[test]
fn cancel_round() {
    //create_project extrinsic
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];
        //schedule_round extrinsic
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 2,
            project_keys.clone(),
            RoundType::ContributionRound,
        )
        .unwrap();

        let round_index = 1;

        //cancel_round extrinsic
        assert_ok!(<proposals::Pallet<Test>>::cancel_round(
            RuntimeOrigin::root(),
            round_index
        ));

        let exp_roundcancelled_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            exp_roundcancelled_event,
            mock::RuntimeEvent::from(proposals::Event::RoundCancelled(1))
        );
    });
}

#[test]
fn test_cancelling_started_round() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let round_key = 1;

        assert_noop!(
            Proposals::cancel_round(RuntimeOrigin::root(), round_key),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::RoundStarted.into(),
            }
        );
    });
}

#[test]
//only user with root privilege can cancel the round
fn test_cancelling_round_without_root_privilege() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();
        let round_key = 1;
        assert_noop!(
            Proposals::cancel_round(RuntimeOrigin::signed(*ALICE), round_key),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: DispatchError::BadOrigin,
            }
        );
    });
}

#[test]
fn create_a_test_project_and_schedule_round_and_contribute() {
    build_test_externality().execute_with(|| {
        //create_project extrinsic
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];
        let project_key: u32 = 0;
        let contribution_amount = 2000u64;

        //schedule_round extrinsic
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let _additional_amount = 10_000;

        run_to_block(4);
        //contribute extrinsic
        Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            None,
            project_key,
            contribution_amount,
        )
        .unwrap();
        Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            None,
            project_key,
            contribution_amount,
        )
        .unwrap();
        Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            None,
            project_key,
            contribution_amount,
        )
        .unwrap();

        //contribute success RuntimeEvent
        let exp_contributedtoproject_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            exp_contributedtoproject_event,
            mock::RuntimeEvent::from(proposals::Event::ContributeSucceeded(
                *ALICE,
                project_key,
                contribution_amount,
                CurrencyId::Native,
                4
            ))
        );
    });
}

#[test]
fn create_a_test_project_and_schedule_round_and_add_whitelist_with_cap_and_contribute() {
    build_test_externality().execute_with(|| {
        //create_project extrinsic
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];
        let project_key: u32 = 0;
        let contribution_amount = 2000u64;
        let max_cap = 1000000u64;

        let whitelist = bounded_btree_map!(*ALICE => max_cap);
        Proposals::add_project_whitelist(RuntimeOrigin::signed(*ALICE), project_key, whitelist)
            .unwrap();

        //schedule_round extrinsic
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let _additional_amount = contribution_amount;

        run_to_block(4);

        //contribute extrinsic
        Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            None,
            project_key,
            contribution_amount,
        )
        .unwrap();

        //contribute success RuntimeEvent
        let exp_contributedtoproject_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            exp_contributedtoproject_event,
            mock::RuntimeEvent::from(proposals::Event::ContributeSucceeded(
                *ALICE,
                project_key,
                contribution_amount,
                CurrencyId::Native,
                4
            ))
        );
    });
}

#[test]
fn create_a_test_project_and_schedule_round_and_add_whitelist_with_unlimited_cap_and_contribute() {
    build_test_externality().execute_with(|| {
        //create_project extrinsic
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];
        let project_key: u32 = 0;
        let contribution_amount = 2000u64;
        let max_cap = 0u64;

        let whitelist = bounded_btree_map!(*ALICE => max_cap);
        Proposals::add_project_whitelist(RuntimeOrigin::signed(*ALICE), project_key, whitelist)
            .unwrap();

        //schedule_round extrinsic
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let _additional_amount = contribution_amount;

        run_to_block(4);

        //contribute extrinsic
        Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            None,
            project_key,
            contribution_amount,
        )
        .unwrap();

        //contribute success RuntimeEvent
        let exp_contributedtoproject_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            exp_contributedtoproject_event,
            mock::RuntimeEvent::from(proposals::Event::ContributeSucceeded(
                *ALICE,
                project_key,
                contribution_amount,
                CurrencyId::Native,
                4
            ))
        );
    });
}

#[test]
fn create_a_test_project_and_schedule_round_and_add_whitelist_and_contribute_over_capfail() {
    build_test_externality().execute_with(|| {
        //create_project extrinsic
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];
        let project_key: u32 = 0;
        let contribution_amount = 60_000u64;
        let max_cap = 100_000u64;

        let whitelist = bounded_btree_map!(*ALICE => max_cap);
        Proposals::add_project_whitelist(RuntimeOrigin::signed(*ALICE), project_key, whitelist)
            .unwrap();

        //schedule_round extrinsic
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        run_to_block(4);
        Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            None,
            project_key,
            contribution_amount,
        )
        .unwrap();

        assert_noop!(
            Proposals::contribute(
                RuntimeOrigin::signed(*ALICE),
                None,
                project_key,
                contribution_amount
            ),
            //approve project
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::ContributionMustBeLowerThanMaxCap.into()
            }
        );
    });
}

#[test]
fn create_a_test_project_and_schedule_round_and_contribute_and_approve() {
    build_test_externality().execute_with(|| {
        //create_project extrinsic
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];
        let project_key = 0;
        let contribution_amount = 1000000u64;

        //schedule_round extrinsic
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let _additional_amount = contribution_amount;

        run_to_block(4);
        //contribute extrinsic
        Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            None,
            project_key,
            contribution_amount,
        )
        .unwrap();

        let project_key = 0;
        //approve project
        Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();

        //approve RuntimeEvent
        let exp_approvedproject_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            exp_approvedproject_event,
            mock::RuntimeEvent::from(proposals::Event::ProjectApproved(1, project_key))
        );
    });
}

#[test]
//negative test case - Approve fails because contribution amount has not met the project required funds
fn create_a_test_project_and_schedule_round_and_contribute_and_approvefail() {
    build_test_externality().execute_with(|| {
        //create_project extrinsic
        assert_ok!(create_project());

        let project_keys: BoundedProjectKeys = bounded_vec![0];
        let project_key = 0;
        let contribution_amount = 100000u64;

        //schedule_round extrinsic
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let _additional_amount = contribution_amount;

        run_to_block(4);
        //contribute extrinsic
        Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            None,
            project_key,
            contribution_amount,
        )
        .unwrap();

        assert_noop!(
            //approve project
            Proposals::approve(RuntimeOrigin::root(), None, project_key, None),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::RoundNotEnded.into()
            }
        );
    });
}

#[test]
fn test_submit_milestone() {
    let voting_round_key = 2;

    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let value = 100u64;
        Proposals::contribute(RuntimeOrigin::signed(*BOB), None, project_key, value).unwrap();

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(0);

        run_to_block(3);

        Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            0
        ));

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::VotingRoundCreated(
                voting_round_key,
                vec![project_key]
            ))
        );
    });
}

#[test]
//negative test case - cannot submit milestones for unapproved projects
fn test_submit_milestone_without_approval() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let value = 100u64;
        assert_ok!(Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            value
        ));

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(0);

        run_to_block(3);

        assert_noop!(
            Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 0),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::OnlyApprovedProjectsCanSubmitMilestones.into(),
            }
        );
    });
}

#[test]
fn test_voting_on_a_milestone() {
    let milestone1_key = 0;
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let value = 100u64;
        Proposals::contribute(RuntimeOrigin::signed(*BOB), None, project_key, value).unwrap();

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(0);

        run_to_block(3);

        assert_ok!(Proposals::approve(
            RuntimeOrigin::root(),
            None,
            project_key,
            None
        ));

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            0
        ));

        run_to_block(5);
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            milestone1_key,
            None,
            true
        ));

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::VoteComplete(*BOB, 0, 0, true, 5))
        );
    });
}

#[test]
//voting on cancelled round should throw error
fn test_voting_on_a_cancelled_round() {
    let round_key = 1;

    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![project_key];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number() + 1,
            System::block_number() + 2,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        assert_ok!(<proposals::Pallet<Test>>::cancel_round(
            RuntimeOrigin::root(),
            round_key
        ));

        run_to_block(5);

        // A strange test as voting on a milestone is not permitted during a contribution round, only a voting round.
        // Todo:? test that contribution is not allowed after the round is cancelled.
        let milestone_key = 0;
        assert_noop!(
            Proposals::vote_on_milestone(
                RuntimeOrigin::signed(*BOB),
                project_key,
                milestone_key,
                None,
                true
            ),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::InvalidRoundType.into(),
            }
        );

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::RoundCancelled(round_key))
        );
    });
}

#[test]
//negative test case where the project creator tries to finalize milestone without getting the vote on that milestone
fn test_finalize_a_milestone_without_voting() {
    let milestone1_key = 0;
    let milestone2_key = 1;
    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
    let milestone1: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 20,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 30,
    };

    let milestone3: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);
    proposed_milestones.push(milestone3);

    build_test_externality().execute_with(|| {
        assert_ok!(create_project_multiple_milestones(proposed_milestones));

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let value = 100u64;
        Proposals::contribute(RuntimeOrigin::signed(*BOB), None, project_key, value).unwrap();

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(0);
        let _ = milestone_index.try_push(1);

        run_to_block(3);

        Proposals::approve(
            RuntimeOrigin::root(),
            None,
            project_key,
            Some(milestone_index),
        )
        .unwrap();

        // Test you can submit a milestone whenever.
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone1_key
        ));

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone2_key
        ));

        run_to_block(5);
        assert_ok!(Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            milestone1_key,
            None,
            true
        ));

        //this works as the voting has been done for this milestone
        assert_ok!(Proposals::finalise_milestone_voting(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            0
        ));

        assert_noop!(
            Proposals::finalise_milestone_voting(RuntimeOrigin::signed(*ALICE), project_key, 1),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::MilestoneVotingNotComplete.into(),
            }
        );
    });
}

#[test]
fn test_project_initiator_cannot_withdraw_if_majority_vote_against() {
    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();

    let milestone1: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 20,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 30,
    };

    let milestone3: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);
    proposed_milestones.push(milestone3);

    build_test_externality().execute_with(|| {
        assert_ok!(create_project_multiple_milestones(proposed_milestones));

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let bob_contribution = 200_000u64;
        assert_ok!(Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            bob_contribution
        ));

        // Second contribution to give Bob majority
        let bob_second_contribution = 400_000u64;
        assert_ok!(Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            bob_second_contribution
        ));

        let charlie_contribution = 500_000u64;
        assert_ok!(Proposals::contribute(
            RuntimeOrigin::signed(*CHARLIE),
            None,
            project_key,
            charlie_contribution
        ));

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(0);
        let _ = milestone_index.try_push(1);

        run_to_block(3);

        assert_ok!(Proposals::approve(
            RuntimeOrigin::root(),
            None,
            project_key,
            None
        ));

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            0
        ));

        run_to_block(5);
        let milestone_key = 0;
        //Bob voting on the submitted milestone
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            milestone_key,
            None,
            false,
        )
        .ok();

        //Charlie voting on the submitted milestone
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*CHARLIE),
            project_key,
            milestone_key,
            None,
            true,
        )
        .ok();

        assert_ok!(Proposals::finalise_milestone_voting(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            0
        ));

        assert_noop!(
            Proposals::withdraw(RuntimeOrigin::signed(*ALICE), project_key),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::NoAvailableFundsToWithdraw.into(),
            }
        );
    })
}

#[test]
fn test_project_initiator_can_withdraw_only_the_percentage_milestone_completed() {
    let additional_amount = 10000000u64;
    let required_funds = 1000000u64;
    let milestone1_key = 0;
    let milestone2_key = 1;
    let milestone3_key = 2;

    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();

    let milestone1: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 20,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 30,
    };

    let milestone3: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);
    proposed_milestones.push(milestone3);
    let proposed_milestones1 = proposed_milestones.clone();

    build_test_externality().execute_with(|| {
        assert_ok!(create_project_multiple_milestones(proposed_milestones));

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let value = 500000u64;
        Proposals::contribute(RuntimeOrigin::signed(*BOB), None, project_key, value).unwrap();

        Proposals::contribute(RuntimeOrigin::signed(*CHARLIE), None, project_key, value).unwrap();

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(milestone1_key);
        let _ = milestone_index.try_push(milestone2_key);

        run_to_block(3);

        Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();

        Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone1_key)
            .unwrap();

        Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, milestone2_key)
            .ok();

        run_to_block(5);
        //Bob voting on the submitted milestone
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            milestone1_key,
            None,
            true,
        )
        .ok();
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            milestone2_key,
            None,
            true,
        )
        .ok();

        //Charlie voting on the submitted milestone
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*CHARLIE),
            project_key,
            milestone1_key,
            None,
            true,
        )
        .ok();
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*CHARLIE),
            project_key,
            milestone2_key,
            None,
            true,
        )
        .ok();

        assert_ok!(Proposals::finalise_milestone_voting(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            0
        ));

        assert_ok!(Proposals::finalise_milestone_voting(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            1
        ));

        assert_ok!(<proposals::Pallet<Test>>::withdraw(
            RuntimeOrigin::signed(*ALICE),
            project_key
        ));

        //calculating the total percentage that can be withdrawn based on the submitted milestones
        let initial_percentage_to_withdraw: u32 =
            proposed_milestones1.get(0).unwrap().percentage_to_unlock
                + proposed_milestones1.get(1).unwrap().percentage_to_unlock;

        //making sure that only balance is equal to the amount withdrawn
        //making sure not all the required funds have been assigned instead only the percentage eligible could be withdrawn
        assert_ne!(
            Tokens::free_balance(CurrencyId::Native, &*ALICE),
            additional_amount + required_funds
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &*ALICE),
            (additional_amount + required_funds * (initial_percentage_to_withdraw as u64) / 100) - <Test as Config>::ProjectStorageDeposit::get()
        );

        // withdraw last milestone
        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            2
        ));
        run_to_block(10);
        //Bob voting on the submitted milestone
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            milestone3_key,
            None,
            true,
        )
        .ok();
        //Charlie voting on the submitted milestone
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*CHARLIE),
            project_key,
            milestone3_key,
            None,
            true,
        )
        .ok();

        assert_ok!(Proposals::finalise_milestone_voting(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone3_key
        ));

        assert_ok!(<proposals::Pallet<Test>>::withdraw(
            RuntimeOrigin::signed(*ALICE),
            project_key
        ));

        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &*ALICE),
            additional_amount + required_funds
        );

        //can withdraw only the amount corresponding to the milestone percentage completion
        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::ProjectFundsWithdrawn(
                *ALICE,
                0,
                500000u64,
                CurrencyId::Native
            ))
        );
    })
}

#[test]
fn test_project_initiator_can_withdraw_only_the_percentage_after_force_milestone_completed() {
    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();

    let milestone1: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 20,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 30,
    };

    let milestone3: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);
    proposed_milestones.push(milestone3);
    let proposed_milestones1 = proposed_milestones.clone();

    build_test_externality().execute_with(|| {
        let initial_balance = Tokens::free_balance(CurrencyId::Native, &ALICE);
        let _required_funds = 1_000_000u64;
        assert_ok!(create_project_multiple_milestones(proposed_milestones));

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![0];

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let contribution_value = 500000u64;
        Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            contribution_value,
        )
        .unwrap();
        Proposals::contribute(
            RuntimeOrigin::signed(*CHARLIE),
            None,
            project_key,
            contribution_value,
        )
        .unwrap();

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(0);
        let _ = milestone_index.try_push(1);

        run_to_block(3);

        Proposals::approve(
            RuntimeOrigin::root(),
            None,
            project_key,
            Some(milestone_index),
        )
        .unwrap();

        assert_ok!(<proposals::Pallet<Test>>::withdraw(
            RuntimeOrigin::signed(*ALICE),
            project_key
        ));

        //calculating the total percentage that can be withdrawn based on the submitted milestones
        let total_percentage_to_withdraw: u32 =
            proposed_milestones1.get(0).unwrap().percentage_to_unlock
                + proposed_milestones1.get(1).unwrap().percentage_to_unlock;

        let project = Projects::<Test>::get(project_key).expect("qed");
        //making sure that only balance is equal to the amount withdrawn
        //making sure not all the required funds have been assigned instead only the percentage eligible could be withdrawn
        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &*ALICE),
            initial_balance + (project.raised_funds * (total_percentage_to_withdraw as u64) / 100) - <Test as Config>::ProjectStorageDeposit::get()
        );

        //can withdraw only the amount corresponding to the milestone percentage completion
        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::ProjectFundsWithdrawn(
                *ALICE,
                0,
                500000u64,
                CurrencyId::Native
            ))
        );
    })
}

#[test]
fn test_withdraw_upon_project_approval_and_finalised_voting() {
    let milestone1_key = 0;
    build_test_externality().execute_with(|| {
        let initial_balance = Tokens::free_balance(CurrencyId::Native, &ALICE);
        assert_ok!(create_project());

        let project_key = 0;
        let project_keys: BoundedProjectKeys = bounded_vec![0];
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let required_funds = 100u64;
        Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            required_funds,
        )
        .unwrap();

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(0);

        run_to_block(3);

        Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();

        Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), project_key, 0).unwrap();

        run_to_block(5);
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            milestone1_key,
            None,
            true,
        )
        .unwrap();

        Proposals::finalise_milestone_voting(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone1_key,
        )
        .unwrap();

        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(*ALICE),
            project_key
        ));

        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &*ALICE),
            initial_balance + required_funds
        );
        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::ProjectFundsWithdrawn(
                *ALICE,
                0,
                100,
                CurrencyId::Native
            ))
        );
    });
}

#[test]
fn test_withdraw_from_non_initiator_account() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        let project_key = 0;

        assert_noop!(
            Proposals::withdraw(RuntimeOrigin::signed(*BOB), project_key),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::InvalidAccount.into(),
            }
        );
    });
}

#[test]
//positive test case submit multiple milestones
fn submit_multiple_milestones() {
    let voting_round1_key = 2;
    let voting_round2_key = 3;
    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
    let milestone1: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 50,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);

    let project_keys: BoundedProjectKeys = bounded_vec![0];

    build_test_externality().execute_with(|| {
        assert_ok!(create_project_multiple_milestones(proposed_milestones));

        let project_key = 0;
        let milestone_index_1 = 0;
        let milestone_index_2 = 1;

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let value = 100u64;
        Proposals::contribute(RuntimeOrigin::signed(*BOB), None, project_key, value).unwrap();

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(milestone_index_1);
        let _ = milestone_index.try_push(milestone_index_2);

        run_to_block(3);

        Proposals::approve(RuntimeOrigin::root(), None, project_key, None).unwrap();

        Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone_index_1,
        )
        .unwrap();

        let voting_round_event_1 = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            voting_round_event_1,
            mock::RuntimeEvent::from(proposals::Event::VotingRoundCreated(
                voting_round1_key,
                vec![project_key]
            ))
        );

        run_to_block(5);

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone_index_2
        ));

        let voting_round_event_2 = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            voting_round_event_2,
            mock::RuntimeEvent::from(proposals::Event::VotingRoundCreated(
                voting_round2_key,
                vec![project_key]
            ))
        );
    });
}

#[test]
fn withdraw_percentage_milestone_completed_refund_locked_milestone() {
    let additional_amount = 10000000u64;
    let required_funds = 1000000u64;
    let project_key = 0;

    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();

    let milestone1: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 20,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 30,
    };

    let milestone3: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);
    proposed_milestones.push(milestone3);
    let proposed_milestones1 = proposed_milestones.clone();

    build_test_externality().execute_with(|| {
        let initial_balance = Tokens::free_balance(CurrencyId::Native, &*ALICE);
        assert_ok!(create_project_multiple_milestones(proposed_milestones));

        let project_keys: BoundedProjectKeys = bounded_vec![0];
        let milestone1_key = 0;
        let milestone2_key = 1;

        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 1,
            project_keys,
            RoundType::ContributionRound,
        )
        .unwrap();

        let contribution_value = 500000u64;
        Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            contribution_value,
        )
        .unwrap();
        Proposals::contribute(
            RuntimeOrigin::signed(*CHARLIE),
            None,
            project_key,
            contribution_value,
        )
        .unwrap();

        //validating contributor current balance
        assert_eq!(
            initial_balance - contribution_value,
            Tokens::free_balance(CurrencyId::Native, &*BOB)
        );
        assert_eq!(
            initial_balance - contribution_value,
            Tokens::free_balance(CurrencyId::Native, &*CHARLIE)
        );

        let mut milestone_index: BoundedMilestoneKeys<Test> = bounded_vec![];
        let _ = milestone_index.try_push(0);

        run_to_block(3);

        Proposals::approve(
            RuntimeOrigin::root(),
            None,
            project_key,
            Some(milestone_index),
        )
        .unwrap();

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone1_key
        ));

        assert_ok!(Proposals::submit_milestone(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            milestone2_key
        ));

        run_to_block(5);
        //Bob voting on the submitted milestone
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*BOB),
            project_key,
            milestone1_key,
            None,
            true,
        )
        .ok();

        //Charlie voting on the submitted milestone
        Proposals::vote_on_milestone(
            RuntimeOrigin::signed(*CHARLIE),
            project_key,
            milestone1_key,
            None,
            true,
        )
        .ok();

        assert_ok!(Proposals::finalise_milestone_voting(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            0
        ));

        assert_ok!(<proposals::Pallet<Test>>::withdraw(
            RuntimeOrigin::signed(*ALICE),
            project_key
        ));

        //calculating the total percentage that can be withdrawn based on the submitted milestones
        let total_percentage_to_withdraw: u32 =
            proposed_milestones1.get(0).unwrap().percentage_to_unlock;

        //making sure that only balance is equal to the amount withdrawn
        //making sure not all the required funds have been assigned instead only the percentage eligible could be withdrawn
        //checking that Alice now has 10.2m
        assert_ne!(
            Tokens::free_balance(CurrencyId::Native, &*ALICE),
            additional_amount + required_funds
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &*ALICE),
            (additional_amount + required_funds * (total_percentage_to_withdraw as u64) / 100) - <Test as Config>::ProjectStorageDeposit::get()
        );

        //can withdraw only the amount corresponding to the milestone percentage completion
        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::ProjectFundsWithdrawn(
                *ALICE,
                project_key,
                200000u64,
                CurrencyId::Native
            ))
        );

        // Call a vote of no confidence and assert it will pass.
        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(*BOB),
            project_key
        ));

        // Charlie has raised a vote of no confidence, now Bob is gonna disagree!
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(*CHARLIE),
            None,
            project_key,
            false
        ));

        assert_ok!(Proposals::finalise_no_confidence_round(
            RuntimeOrigin::signed(*CHARLIE),
            None,
            project_key
        ));

        let approved_milestone_value = 100000;
        //ensuring the refunded amount was transferred back successfully
        assert_eq!(
            initial_balance - approved_milestone_value,
            Tokens::free_balance(CurrencyId::Native, &*BOB)
        );
        assert_eq!(
            initial_balance - approved_milestone_value,
            Tokens::free_balance(CurrencyId::Native, &*CHARLIE)
        );
    })
}

#[test]
fn test_schedule_round_fails_gracefully_with_empty_vec() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());

        assert_noop!(
            Proposals::schedule_round(
                RuntimeOrigin::root(),
                System::block_number(),
                System::block_number() + 1,
                // Empty keys is the test.
                bounded_vec![],
                RoundType::ContributionRound
            ),
            Error::<Test>::LengthMustExceedZero
        );
    });
}

#[test]
fn test_raising_a_vote_of_no_confidence() {
    let project_key = 0u32;

    build_test_externality().execute_with(|| {
        // Create a project for both ALICE and BOB.
        assert_ok!(create_project());

        // Schedule a round to allow for contributions.
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![project_key],
            RoundType::ContributionRound,
        )
        .unwrap();

        // Deposit funds and contribute.
        run_to_block(System::block_number() + 3);

        Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key,
            1_000_000u64,
        )
        .unwrap();
        run_to_block(System::block_number() + 101);

        Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();

        // Assert that Bob cannot raise the vote as he is not a contributor.
        assert_noop!(
            Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*CHARLIE), project_key),
            Error::<Test>::OnlyContributorsCanVote
        );

        // Call a vote of no confidence and assert it will pass.
        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(*BOB),
            project_key
        ));

        let vote = NoConfidenceVotes::<Test>::get(project_key).unwrap();
        let round_count = RoundCount::<Test>::get();

        // Assert that storage has been mutated correctly.
        assert!(vote.nay == 1_000_000u64 && vote.yay == 0u64);
        assert!(UserVotes::<Test>::get((*BOB, project_key, 0, round_count)) == Some(true));
        assert!(round_count == 2u32);
        assert!(NoConfidenceVotes::<Test>::contains_key(project_key));

        // Assert that you cannot raise the vote twice.
        assert_noop!(
            Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*BOB), project_key),
            Error::<Test>::RoundStarted
        );
    });
}

#[test]
fn test_adding_vote_of_no_confidence() {
    let project_key = 0u32;
    build_test_externality().execute_with(|| {
        // Create a project for both ALICE and BOB.
        assert_ok!(create_project());

        //schedule a round to allow for contributions.
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![project_key],
            RoundType::ContributionRound,
        )
        .unwrap();

        // Deposit funds and contribute.
        run_to_block(System::block_number() + 3);

        // Setup required state to start voting: must have contributed and round must have started.
        Proposals::contribute(
            RuntimeOrigin::signed(*CHARLIE),
            Some(1),
            project_key,
            500_000u64,
        )
        .unwrap();
        Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key,
            500_000u64,
        )
        .unwrap();

        run_to_block(System::block_number() + 101);

        // Assert that threshold has been met
        assert_ok!(Proposals::approve(
            RuntimeOrigin::root(),
            Some(1),
            project_key,
            None
        ));

        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(*CHARLIE),
            project_key
        ));

        // Charlie has raised a vote of no confidence, now Bob is gonna disagree!
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            true
        ));

        // Assert Bob cannot game the system.
        assert_noop!(
            Proposals::vote_on_no_confidence_round(
                RuntimeOrigin::signed(*BOB),
                None,
                project_key,
                true
            ),
            Error::<Test>::VoteAlreadyExists
        );
        assert_noop!(
            Proposals::vote_on_no_confidence_round(
                RuntimeOrigin::signed(*BOB),
                None,
                project_key,
                false
            ),
            Error::<Test>::VoteAlreadyExists
        );

        // Assert the state of the system is as it should be.
        let vote = NoConfidenceVotes::<Test>::get(project_key).unwrap();
        let round_count = RoundCount::<Test>::get();

        // Assert that storage has been mutated correctly.
        assert!(vote.nay == 500_000u64 && vote.yay == 500_000u64);
        assert!(UserVotes::<Test>::get((*CHARLIE, project_key, 0, round_count)) == Some(true));
        assert!(UserVotes::<Test>::get((*BOB, project_key, 0, round_count)) == Some(true));

        assert!(round_count == 2u32);
    });
}

#[test]
fn test_finalise_vote_of_no_confidence_with_threshold_met() {
    let project_key = 0u32;
    build_test_externality().execute_with(|| {
        // Create a project for both ALICE and BOB.
        assert_ok!(create_project());

        //schedule a round to allow for contributions.
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![project_key],
            RoundType::ContributionRound,
        )
        .unwrap();

        // Deposit funds and contribute.
        run_to_block(System::block_number() + 3);
        // Setup required state to start voting: must have contributed and round must have started.
        Proposals::contribute(
            RuntimeOrigin::signed(*CHARLIE),
            Some(1),
            project_key,
            750_001u64,
        )
        .unwrap();
        Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key,
            250_000u64,
        )
        .unwrap();
        run_to_block(System::block_number() + 101);
        Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();

        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(*CHARLIE),
            project_key
        ));
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            false
        ));

        // Assert that steve who is not a contributor cannot finalise the same goes for the initiator.
        assert_noop!(
            Proposals::finalise_no_confidence_round(
                RuntimeOrigin::signed(*ALICE),
                None,
                project_key
            ),
            Error::<Test>::OnlyContributorsCanVote
        );
        assert_noop!(
            Proposals::finalise_no_confidence_round(
                RuntimeOrigin::signed(*ALICE),
                None,
                project_key
            ),
            Error::<Test>::OnlyContributorsCanVote
        );
        // And we might aswell assert that you cannot call finalise on a project key that doesnt exist.
        assert_noop!(
            Proposals::finalise_no_confidence_round(RuntimeOrigin::signed(*BOB), None, 2),
            Error::<Test>::ProjectNotInRound
        );
        // Assert that BOB, a contrbutor, can finalise
        assert_ok!(Proposals::finalise_no_confidence_round(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key
        ));
    });
}

// I Realised that i have already tested for thresholds on the mark and therefore above
// Alas i should test below the threshold
#[test]
fn test_finalise_vote_of_no_confidence_below_threshold() {
    let project_key = 0u32;
    build_test_externality().execute_with(|| {
        // Create a project for both ALICE and BOB.
        assert_ok!(create_project());

        //schedule a round to allow for contributions.
        Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![project_key],
            RoundType::ContributionRound,
        )
        .unwrap();

        // Deposit funds and contribute.
        run_to_block(System::block_number() + 3);

        // Setup required state to start voting: must have contributed and round must have started.
        Proposals::contribute(
            RuntimeOrigin::signed(*CHARLIE),
            Some(1),
            project_key,
            500_000u64,
        )
        .unwrap();
        Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key,
            500_000u64,
        )
        .unwrap();

        run_to_block(System::block_number() + 101);

        // Assert that threshold has been met
        assert_ok!(Proposals::approve(
            RuntimeOrigin::root(),
            Some(1),
            project_key,
            None
        ));

        assert_ok!(Proposals::raise_vote_of_no_confidence(
            RuntimeOrigin::signed(*CHARLIE),
            project_key
        ));
        assert_ok!(Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(*BOB),
            Some(2),
            project_key,
            true
        ));

        assert_noop!(
            Proposals::finalise_no_confidence_round(
                RuntimeOrigin::signed(*CHARLIE),
                Some(2),
                project_key
            ),
            Error::<Test>::VoteThresholdNotMet
        );
    });
}

#[test]
fn test_finalise_vote_of_no_confidence_refunds_contributors() {
    // The project creator.

    // The contributors.

    build_test_externality().execute_with(|| {
        let initial_balance = Tokens::free_balance(CurrencyId::Native, &*BOB);
        let project_key = 0u32;
        // Create a project for both ALICE and BOB.
        assert_ok!(create_project());

        let _ = Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![project_key],
            RoundType::ContributionRound,
        )
        .unwrap();
        run_to_block(System::block_number() + 3);
        let _ = Proposals::contribute(
            RuntimeOrigin::signed(*CHARLIE),
            Some(1),
            project_key,
            750_000u64,
        )
        .unwrap();
        let _ = Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key,
            250_000u64,
        )
        .unwrap();
        run_to_block(System::block_number() + 101);

        // assert that the voters have had their funds transferred.
        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &BOB),
            initial_balance - 250_000u64
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &CHARLIE),
            initial_balance - 750_000
        );

        // approve and raise votees
        let _ = Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, None).unwrap();
        let _ =
            Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*CHARLIE), project_key)
                .unwrap();
        let _ = Proposals::vote_on_no_confidence_round(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key,
            false,
        )
        .unwrap();

        // Assert that BOB, a contrbutor, can finalise
        assert_ok!(Proposals::finalise_no_confidence_round(
            RuntimeOrigin::signed(*BOB),
            None,
            project_key
        ));

        // Wait a block so that refunds occur;
        run_to_block(System::block_number() + 1);
        // assert that the voters have had their funds refunded.
        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &CHARLIE),
            initial_balance
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &BOB),
            initial_balance
        );
    });
}

// create project, schedule a round, approve and submit a milestone.
// assert that the vote will pass when it is on the threshold.
#[test]
fn test_finalise_milestone_is_ok_on_threshold_vote() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());
        let project_key = 0;
        let round_id = 1;

        let _ = Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![0u32],
            RoundType::ContributionRound,
        )
        .unwrap();

        let yes_contribution = 1_000_000u64 / 100u64 * PercentRequiredForVoteToPass::get() as u64;
        let no_contribution =
            1_000_000u64 / 100u64 * (100u8 - PercentRequiredForVoteToPass::get()) as u64;

        run_to_block(System::block_number() + 1);

        let _ = Proposals::contribute(
            RuntimeOrigin::signed(*ALICE),
            Some(round_id),
            project_key,
            yes_contribution,
        )
        .unwrap();
        let _ = Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(round_id),
            project_key,
            no_contribution,
        )
        .unwrap();

        run_to_block(System::block_number() + 100);

        // Assert that threshold has been met
        let _ = Proposals::approve(RuntimeOrigin::root(), Some(1), 0, None).unwrap();

        let _ = Proposals::submit_milestone(RuntimeOrigin::signed(*ALICE), 0, 0).unwrap();

        run_to_block(System::block_number() + 1);

        let _ =
            Proposals::vote_on_milestone(RuntimeOrigin::signed(*ALICE), 0, 0, None, true).unwrap();
        let _ =
            Proposals::vote_on_milestone(RuntimeOrigin::signed(*BOB), 0, 0, None, false).unwrap();

        assert_ok!(Proposals::finalise_milestone_voting(
            RuntimeOrigin::signed(*ALICE),
            0,
            0
        ));
    })
}

#[test]
// update project required funds and milestones - positive test case
fn update_an_existing_project() {
    let updated_required_funds = 2_500_000u64;
    let updated_agreement_hash = gen_hash(200);
    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
    let milestone1: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 20,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 30,
    };

    let milestone3: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);
    proposed_milestones.push(milestone3);

    let mut updated_proposed_milestones: Vec<ProposedMilestone> = Vec::new();
    let updated_milestone1: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 70,
    };
    let updated_milestone2: ProposedMilestone = ProposedMilestone {
        percentage_to_unlock: 30,
    };

    updated_proposed_milestones.push(updated_milestone1);
    updated_proposed_milestones.push(updated_milestone2);

    build_test_externality().execute_with(|| {
        assert_ok!(create_project_multiple_milestones(proposed_milestones));

        let project_key = 0;

        assert_ok!(Proposals::update_project(
            RuntimeOrigin::signed(*ALICE),
            project_key,
            updated_proposed_milestones
                .try_into()
                .expect("Invalid proposed milestones"),
            updated_required_funds,
            CurrencyId::Native,
            gen_hash(200)
        ));

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(proposals::Event::ProjectUpdated(
                *ALICE,
                project_key,
                updated_required_funds
            ))
        );

        let updated_project = Projects::<Test>::get(&project_key).unwrap();

        assert_eq!(updated_project.required_funds, updated_required_funds);
        assert_eq!(updated_project.agreement_hash, updated_agreement_hash);
    });
}

#[test]
fn only_the_initiator_can_update_project() {
    build_test_externality().execute_with(|| {
        assert_ok!(create_project());
        let project_key = 0;
        let updated_funds = 1_000;
        let updated_milestone1: ProposedMilestone = ProposedMilestone {
            percentage_to_unlock: 70,
        };

        let updated_milestone2: ProposedMilestone = ProposedMilestone {
            percentage_to_unlock: 30,
        };

        assert_noop!(
            Proposals::update_project(
                RuntimeOrigin::signed(*BOB),
                project_key,
                vec![updated_milestone1.clone()].try_into().expect("qed"),
                updated_funds,
                CurrencyId::Native,
                gen_hash(1),
            ),
            Error::<Test>::MilestonesTotalPercentageMustEqual100
        );

        assert_noop!(
            Proposals::update_project(
                RuntimeOrigin::signed(*BOB),
                project_key,
                vec![updated_milestone1, updated_milestone2]
                    .try_into()
                    .expect("qed"),
                updated_funds,
                CurrencyId::Native,
                gen_hash(1),
            ),
            Error::<Test>::UserIsNotInitiator
        );
    })
}

#[test]
fn deposit_taken_on_project_creation() {
    build_test_externality().execute_with(|| {
        let alice_initial = Tokens::free_balance(CurrencyId::Native, &ALICE);
        let _ = create_project();
        let alice_after = Tokens::free_balance(CurrencyId::Native, &ALICE);

        assert_eq!(alice_after + <Test as Config>::ProjectStorageDeposit::get(), alice_initial);
    })
}

#[test]
fn project_is_deleted_on_final_withdraw() {
    build_test_externality().execute_with(|| {
        let _ = create_project();
    })
}


#[test]
fn project_is_deleted_after_no_confidence_call() {
    build_test_externality().execute_with(|| {

        let _ = create_project();
        let project_key: ProjectKey = 0;

        let _ = Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![0u32],
            RoundType::ContributionRound,
        );
        let _ = Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key,
            1_000_000u64,
        );
        run_to_block(System::block_number() + 100);
        let _ = Proposals::approve(RuntimeOrigin::root(), Some(1), 0, None).unwrap();
        let _ =
        Proposals::raise_vote_of_no_confidence(RuntimeOrigin::signed(*BOB), project_key)
            .unwrap();

        assert_ok!(Proposals::finalise_no_confidence_round(
                RuntimeOrigin::signed(*BOB),
                None,
                project_key
        ));
        assert!(Projects::<Test>::get(project_key).is_none());
    })
}

//common helper methods
pub fn create_project() -> DispatchResultWithPostInfo {
    Proposals::create_project(
        RuntimeOrigin::signed(*ALICE),
        gen_hash(1),
        bounded_vec![ProposedMilestone {
            percentage_to_unlock: 100
        }],
        //funds required
        1_000_000u64,
        CurrencyId::Native,
    )
}

pub fn create_project_multiple_milestones(
    proposed_milestones: Vec<ProposedMilestone>,
) -> DispatchResultWithPostInfo {
    Proposals::create_project(
        RuntimeOrigin::signed(*ALICE),
        gen_hash(1),
        proposed_milestones
            .try_into()
            .expect("proposed milestones are too long"),
        //funds required
        1_000_000u64,
        CurrencyId::Native,
    )
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Proposals::on_initialize(System::block_number());
        //Bad case scenario is that we have little space. all tests must still pass.
        if n % 2 == 0 {
            Proposals::on_idle(System::block_number(), Weight::MAX / 90);
        } else {
            Proposals::on_idle(System::block_number(), Weight::MAX / 2);
        }
    }
}

fn _run_to_block_with_no_idle_space(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Proposals::on_initialize(System::block_number());
        Proposals::on_idle(System::block_number(), Weight::zero());
    }
}

fn gen_hash(seed: u8) -> AgreementHash {
    H256::from([seed; 32])
}
