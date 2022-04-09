// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate as proposals;
use crate::mock::*;
use crate::*;
use common_types::CurrencyId;
use frame_support::{
    assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo, weights::PostDispatchInfo,
};
use sp_core::sr25519;
use sp_std::str;
use sp_std::vec::Vec;

#[test]
fn create_a_test_project() {
    let mut t = sp_io::TestExternalities::default();
    t.execute_with(|| {
        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        Proposals::create_project(
            Origin::signed(alice),
            //project name
            str::from_utf8(b"Imbue's Awesome Initiative").unwrap().as_bytes().to_vec(),
            //project logo
            str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec(),
            //project description
            str::from_utf8(b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.").unwrap().as_bytes().to_vec(),
            //website
            str::from_utf8(b"https://imbue.network").unwrap().as_bytes().to_vec(),
            //milestone
            vec![ProposedMilestone {
                name: Vec::new(),
                percentage_to_unlock: 100,
            }],
            //funds required
            1000000u64,
            CurrencyId::Native
        ).unwrap();
    });
}

#[test]
fn create_a_test_project_with_less_than_100_percent() {
    let mut t = sp_io::TestExternalities::default();
    t.execute_with(|| {
        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        assert_noop!(
        Proposals::create_project(
            Origin::signed(alice),
            //project name
            str::from_utf8(b"Imbue's Awesome Initiative").unwrap().as_bytes().to_vec(),
            //project logo
            str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec(),
            //project description
            str::from_utf8(b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.").unwrap().as_bytes().to_vec(), 
            //website
            str::from_utf8(b"https://imbue.network").unwrap().as_bytes().to_vec(),
            //milestone
            vec![ProposedMilestone {
                name: Vec::new(), percentage_to_unlock: 99
            }],
            //funds required
            1000000u64,
            CurrencyId::Native
        ),DispatchErrorWithPostInfo {
            post_info: PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::Yes,
            },
            error: Error::<Test>::MilestonesTotalPercentageMustEqual100.into()
        });
    });
}

#[test]
fn create_a_test_project_with_no_name() {
    let mut t = sp_io::TestExternalities::default();

    t.execute_with(|| {
        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        assert_noop!(
        Proposals::create_project(
            Origin::signed(alice),
            //project name
            str::from_utf8(b"").unwrap().as_bytes().to_vec(),
            //project logo
            str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec(),
            //project description
            str::from_utf8(b"This project is aimed at promoting Decentralised Data and Transparent Crowdfunding.").unwrap().as_bytes().to_vec(), 
            //website
            str::from_utf8(b"https://imbue.network").unwrap().as_bytes().to_vec(),
            //milestone
            vec![ProposedMilestone {
                name: Vec::new(), percentage_to_unlock: 99
            }],
            //funds required
            1000000u64,
            CurrencyId::Native
        ),DispatchErrorWithPostInfo {
            post_info: PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::Yes,
            },
            error: Error::<Test>::ProjectNameIsMandatory.into()
        });
    });
}

#[test]
fn create_a_test_project_with_no_data() {
    let mut t = sp_io::TestExternalities::default();
    t.execute_with(|| {
        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        assert_noop!(
            Proposals::create_project(
                Origin::signed(alice),
                //project name
                str::from_utf8(b"").unwrap().as_bytes().to_vec(),
                //project logo
                str::from_utf8(b"").unwrap().as_bytes().to_vec(),
                //project description
                str::from_utf8(b"").unwrap().as_bytes().to_vec(),
                //website
                str::from_utf8(b"").unwrap().as_bytes().to_vec(),
                //milestone
                vec![ProposedMilestone {
                    name: Vec::new(),
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
                error: Error::<Test>::ProjectNameIsMandatory.into()
            }
        );
    });
}

#[test]
fn create_a_test_project_and_schedule_round() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let mut t = sp_io::TestExternalities::default();
    t.execute_with(|| {
        create_project(alice);

        Proposals::schedule_round(
            Origin::root(),
            System::block_number(),
            System::block_number() + 1,
            //Project key starts with 0 for the first project submitted to the chain
            vec![0],
        )
        .unwrap();
    });
}

#[test]
fn schedule_round_invalid_project_key() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let mut t = sp_io::TestExternalities::default();
    t.execute_with(|| {
        create_project(alice);

        assert_noop!(
            Proposals::schedule_round(
                Origin::root(),
                System::block_number(),
                System::block_number() + 1,
                //Project key starts with 0 for the first project submitted to the chain
                vec![1]
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
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let mut t = sp_io::TestExternalities::default();
    t.execute_with(|| {
        create_project(alice);

        assert_noop!(
            Proposals::schedule_round(
                Origin::root(),
                System::block_number() + 6000,
                System::block_number() + 3000,
                //Project key starts with 0 for the first project submitted to the chain
                vec![1]
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
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let mut t = sp_io::TestExternalities::default();
    t.execute_with(|| {
        create_project(alice);

        assert_noop!(
            Proposals::schedule_round(
                Origin::root(),
                System::block_number() + 6000,
                System::block_number() + 3000,
                //Project key starts with 0 for the first project submitted to the chain
                vec![1]
            ),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::EndTooEarly.into()
            }
        );

        assert_noop!(
            Proposals::cancel_round(Origin::root(), 0),
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
fn cancel_round() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    //create_project extrinsic
    ExtBuilder.build().execute_with(|| {
        create_project(alice);
        let project_key: Vec<ProjectKey> = vec![0];
        //schedule_round extrinsic
        assert_ok!(Proposals::schedule_round(
            Origin::root(),
            System::block_number() + 1,
            System::block_number() + 2,
            project_key
        ));

        let exp_fundingroundcreated_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;

        assert_eq!(
            exp_fundingroundcreated_event,
            mock::Event::from(proposals::Event::FundingRoundCreated(0))
        );

        let round_index = 0;

        //cancel_round extrinsic
        assert_ok!(<proposals::Pallet<Test>>::cancel_round(
            Origin::root(),
            round_index
        ));

        let exp_roundcancelled_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            exp_roundcancelled_event,
            mock::Event::from(proposals::Event::RoundCancelled(0))
        );
    });
}


#[test]
fn test_canceling_started_round() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project(alice);

        let project_keys: Vec<ProjectKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1 ,
            System::block_number() + 1,
            project_keys
        ));

        assert_noop!(<proposals::Pallet<Test>>::cancel_round(
            Origin::root(),
            0
        ),DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::RoundStarted.into(),
            });

    });
}

#[test]
//only user with root privilege can cancel the round
fn test_canceling_round_without_root_privilege() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project(alice);

        let project_keys: Vec<ProjectKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1 ,
            System::block_number() + 1,
            project_keys
        ));

        assert_noop!(<proposals::Pallet<Test>>::cancel_round(
            Origin::signed(alice),
            0
        ),DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: DispatchError::BadOrigin,
            });

    });
}





#[test]
fn create_a_test_project_and_schedule_round_and_contribute() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    ExtBuilder.build().execute_with(|| {
        //create_project extrinsic
        create_project(alice);

        let project_keys: Vec<ProjectKey> = vec![0];
        let project_key: u32 = 0;
        let contribution_amount = 2000u64;

        //schedule_round extrinsic
        Proposals::schedule_round(
            Origin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
        )
        .unwrap();

        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        let additional_amount = 10_000;

        let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);

        run_to_block(4);
        //contribute extrinsic
        Proposals::contribute(
            Origin::signed(alice),
            project_key,
            CurrencyId::Native,
            contribution_amount,
        )
        .unwrap();

        //contribute success event
        let exp_contributedtoproject_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            exp_contributedtoproject_event,
            mock::Event::from(proposals::Event::ContributeSucceeded(
                alice,
                project_key,
                contribution_amount,
                CurrencyId::Native,
                4
            ))
        );
    });
}

#[test]
fn create_a_test_project_and_schedule_round_and_contribute_and_approve() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    ExtBuilder.build().execute_with(|| {
        //create_project extrinsic
        create_project(alice);

        let project_keys: Vec<ProjectKey> = vec![0];
        let project_key = 0;
        let milestone_keys: Vec<MilestoneKey> = vec![0];
        let contribution_amount = 1000000u64;

        //schedule_round extrinsic
        Proposals::schedule_round(
            Origin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
        )
        .unwrap();

        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        let additional_amount = 1000000;
        let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);

        run_to_block(4);
        //contribute extrinsic
        Proposals::contribute(
            Origin::signed(alice),
            project_key,
            CurrencyId::Native,
            contribution_amount,
        )
        .unwrap();

        //approve project
        Proposals::approve(Origin::root(), 0, milestone_keys).unwrap();

        //approve event
        let exp_approvedproject_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            exp_approvedproject_event,
            mock::Event::from(proposals::Event::ProjectApproved(1, project_key))
        );
    });
}

#[test]
//negative test case - Approve fails because contribution amount has not met the project required funds
fn create_a_test_project_and_schedule_round_and_contribute_and_approvefail() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    ExtBuilder.build().execute_with(|| {
        //create_project extrinsic
        create_project(alice);

        let project_keys: Vec<ProjectKey> = vec![0];
        let project_key = 0;
        let contribution_amount = 100000u64;
        let milestone_keys: Vec<MilestoneKey> = vec![0];

        //schedule_round extrinsic
        Proposals::schedule_round(
            Origin::root(),
            System::block_number() + 1,
            System::block_number() + 10,
            //Project key starts with 0 for the first project submitted to the chain
            project_keys,
        )
        .unwrap();

        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        let additional_amount = 1000000;
        let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);

        run_to_block(4);
        //contribute extrinsic
        Proposals::contribute(
            Origin::signed(alice),
            project_key,
            CurrencyId::Native,
            contribution_amount,
        )
        .unwrap();

        assert_noop!(
            //approve project
            Proposals::approve(Origin::root(), project_key, milestone_keys),
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
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project(alice);

        let project_index = 0;
        let project_keys: Vec<ProjectKey> = vec![0];
        let milestone_keys: Vec<MilestoneKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1,
            System::block_number() + 1,
            project_keys
        ));

        let value = 100u64;
        assert_ok!(<proposals::Pallet<Test>>::contribute(
            Origin::signed(bob),
            project_index,
            CurrencyId::Native,
            value
        ));

        let mut milestone_index: Vec<MilestoneKey> = Vec::new();
        milestone_index.push(0);

        run_to_block(3);

        assert_ok!(Proposals::approve(
            Origin::root(),
            project_index,
            milestone_keys
        ));

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            0
        ));

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::Event::from(proposals::Event::VotingRoundCreated(1))
        );
    });
}

#[test]
//negative test case - cannot submit milestones for unapproved projects
fn test_submit_milestone_without_approval() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project(alice);

        let project_index = 0;
        let project_keys: Vec<ProjectKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1,
            System::block_number() + 1,
            project_keys
        ));

        let value = 100u64;
        assert_ok!(<proposals::Pallet<Test>>::contribute(
            Origin::signed(bob),
            project_index,
            CurrencyId::Native,
            value
        ));

        let mut milestone_index: Vec<MilestoneKey> = Vec::new();
        milestone_index.push(0);

        run_to_block(3);

        assert_noop!(
            Proposals::submit_milestone(Origin::signed(alice), project_index, 0),
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
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project(alice);

        let project_index = 0;
        let project_keys: Vec<ProjectKey> = vec![0];
        let milestone_keys: Vec<MilestoneKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1,
            System::block_number() + 1,
            project_keys
        ));

        let value = 100u64;
        assert_ok!(<proposals::Pallet<Test>>::contribute(
            Origin::signed(bob),
            project_index,
            CurrencyId::Native,
            value
        ));

        let mut milestone_index: Vec<MilestoneKey> = Vec::new();
        milestone_index.push(0);

        run_to_block(3);

        assert_ok!(Proposals::approve(
            Origin::root(),
            project_index,
            milestone_keys
        ));

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            0
        ));

        run_to_block(5);
        assert_ok!(Proposals::vote_on_milestone(
            Origin::signed(bob),
            project_index,
            0,
            true
        ));

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::Event::from(proposals::Event::VoteComplete(bob, 0, 0, true, 5))
        );
    });
}

#[test]
//voting on canceled round should throw error
fn test_voting_on_a_canceled_round() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project(alice);

        let project_index = 0;
        let project_keys: Vec<ProjectKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() + 1 ,
            System::block_number() + 2,
            project_keys
        ));

        assert_ok!(<proposals::Pallet<Test>>::cancel_round(
            Origin::root(),
            0
        ));

        run_to_block(5);
        assert_noop!(Proposals::vote_on_milestone(
            Origin::signed(bob),
            project_index,
            0,
            true
        ), DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::RoundNotProcessing.into(),
            });

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::Event::from(proposals::Event::RoundCancelled(0))
        );
    });
}



#[test]
//negative test case where the project creator tries to finalize milestone without getting the vote on that milestone
fn test_finalize_a_milestone_without_voting() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
    let milestone1: ProposedMilestone = ProposedMilestone {
        name: str::from_utf8(b"milestone 1").unwrap().as_bytes().to_vec(),
        percentage_to_unlock: 20,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        name: str::from_utf8(b"milestone 2").unwrap().as_bytes().to_vec(),
        percentage_to_unlock: 30,
    };

    let milestone3: ProposedMilestone = ProposedMilestone {
        name: str::from_utf8(b"milestone 3").unwrap().as_bytes().to_vec(),
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);
    proposed_milestones.push(milestone3);

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project_multiple_milestones(alice, proposed_milestones);

        let project_index = 0;
        let project_keys: Vec<ProjectKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1,
            System::block_number() + 1,
            project_keys
        ));

        let value = 100u64;
        assert_ok!(<proposals::Pallet<Test>>::contribute(
            Origin::signed(bob),
            project_index,
            CurrencyId::Native,
            value
        ));

        let mut milestone_index: Vec<MilestoneKey> = Vec::new();
        milestone_index.push(0);
        milestone_index.push(1);

        run_to_block(3);

        assert_ok!(Proposals::approve(
            Origin::root(),
            project_index,
            milestone_index
        ));

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            0
        ));

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            1
        ));

        run_to_block(5);
        assert_ok!(Proposals::vote_on_milestone(
            Origin::signed(bob),
            project_index,
            0,
            true
        ));

        //this works as the voting has been done for this milestone
        assert_ok!(Proposals::finalise_milestone_voting(
            Origin::signed(alice),
            project_index,
            0
        ));

        assert_noop!(Proposals::finalise_milestone_voting(
            Origin::signed(alice),
            project_index,
            1
        ), DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: None,
                    pays_fee: Pays::Yes,
                },
                error: Error::<Test>::MilestoneVotingNotComplete.into(),
            });
    });
}


#[test]
fn test_project_initiator_can_withdraw_only_the_percentage_milestone_completed() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
    let additional_amount = 10000000u64;
    let required_funds = 1000000u64;

    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
    let mut proposed_milestones1: Vec<ProposedMilestone> = Vec::new();

    let milestone1: ProposedMilestone = ProposedMilestone {
        name: str::from_utf8(b"milestone 1").unwrap().as_bytes().to_vec(),
        percentage_to_unlock: 20,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        name: str::from_utf8(b"milestone 2").unwrap().as_bytes().to_vec(),
        percentage_to_unlock: 30,
    };

    let milestone3: ProposedMilestone = ProposedMilestone {
        name: str::from_utf8(b"milestone 3").unwrap().as_bytes().to_vec(),
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);
    proposed_milestones.push(milestone3);
    proposed_milestones1 = proposed_milestones.clone();

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        let _ = Currencies::deposit(CurrencyId::Native, &charlie, additional_amount);
        create_project_multiple_milestones(alice, proposed_milestones);

        let project_index = 0;
        let project_keys: Vec<ProjectKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1,
            System::block_number() + 1,
            project_keys
        ));

        let value = 500000u64;
        assert_ok!(<proposals::Pallet<Test>>::contribute(
            Origin::signed(bob),
            project_index,
            CurrencyId::Native,
            value
        ));

        assert_ok!(<proposals::Pallet<Test>>::contribute(
            Origin::signed(charlie),
            project_index,
            CurrencyId::Native,
            value
        ));

        let mut milestone_index: Vec<MilestoneKey> = Vec::new();
        milestone_index.push(0);
        milestone_index.push(1);

        run_to_block(3);

        assert_ok!(Proposals::approve(
            Origin::root(),
            project_index,
            milestone_index
        ));

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            0
        ));

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            1
        ));

        run_to_block(5);
        //Bob voting on the submitted milestone
        Proposals::vote_on_milestone(Origin::signed(bob),project_index, 0, true,);
        Proposals::vote_on_milestone(Origin::signed(bob),project_index, 1, true,);

        //Charlie voting on the submitted milestone
        Proposals::vote_on_milestone(Origin::signed(charlie),project_index, 0, true,);
        Proposals::vote_on_milestone(Origin::signed(charlie),project_index, 1, true,);

        assert_ok!(Proposals::finalise_milestone_voting(
            Origin::signed(alice),
            project_index,
            0
        ));

        assert_ok!(Proposals::finalise_milestone_voting(
            Origin::signed(alice),
            project_index,
            1
        ));

        assert_ok!(<proposals::Pallet<Test>>::withdraw(
            Origin::signed(alice),
            project_index
        ));

        //calculating the total percentage that can be withdrawn based on the submitted milestones
        let mut total_percentage_to_withdraw:u32 = proposed_milestones1.get(0).unwrap().percentage_to_unlock +  proposed_milestones1.get(1).unwrap().percentage_to_unlock;

        //making sure that only balance is equal to the amount withdrawn
        //making sure not all the required funds have been assigned instead only the percentage eligible could be withdrawn
        assert_ne!(Balances::free_balance(&alice), additional_amount + required_funds);
        assert_eq!(Balances::free_balance(&alice), additional_amount + required_funds * (total_percentage_to_withdraw as u64)/100);

        //can withdraw only the amount corresponding to the milestone percentage completion
        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::Event::from(proposals::Event::ProjectFundsWithdrawn(alice, 0, 500000u64,CurrencyId::Native))
        );

    })
}




#[test]
fn test_withdraw_upon_project_approval_and_finalised_voting() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project(alice);

        let project_index = 0;
        let project_keys: Vec<ProjectKey> = vec![0];
        let milestone_keys: Vec<MilestoneKey> = vec![0];

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1,
            System::block_number() + 1,
            project_keys
        ));

        let required_funds = 100u64;
        assert_ok!(<proposals::Pallet<Test>>::contribute(
            Origin::signed(bob),
            project_index,
            CurrencyId::Native,
            required_funds
        ));

        let mut milestone_index: Vec<MilestoneKey> = Vec::new();
        milestone_index.push(0);

        run_to_block(3);

        assert_ok!(Proposals::approve(
            Origin::root(),
            project_index,
            milestone_keys
        ));

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            0
        ));

        run_to_block(5);
        assert_ok!(Proposals::vote_on_milestone(
            Origin::signed(bob),
            project_index,
            0,
            true
        ));

        assert_ok!(Proposals::finalise_milestone_voting(
            Origin::signed(alice),
            project_index,
            0
        ));

        assert_ok!(<proposals::Pallet<Test>>::withdraw(
            Origin::signed(alice),
            project_index
        ));

        assert_eq!(Balances::free_balance(&alice), additional_amount + required_funds);
        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            latest_event,
            mock::Event::from(proposals::Event::ProjectFundsWithdrawn(
                alice,
                0,
                100,
                CurrencyId::Native
            ))
        );
    });
}

#[test]
fn test_withdraw_from_non_initiator_account() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project(alice);

        let project_index = 0;

        assert_noop!(
            Proposals::withdraw(Origin::signed(bob), project_index),
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
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 100000000u64;

    let mut proposed_milestones: Vec<ProposedMilestone> = Vec::new();
    let milestone1: ProposedMilestone = ProposedMilestone {
        name: str::from_utf8(b"milestone 1").unwrap().as_bytes().to_vec(),
        percentage_to_unlock: 50,
    };
    let milestone2: ProposedMilestone = ProposedMilestone {
        name: str::from_utf8(b"milestone 2").unwrap().as_bytes().to_vec(),
        percentage_to_unlock: 50,
    };
    proposed_milestones.push(milestone1);
    proposed_milestones.push(milestone2);

    let project_keys: Vec<ProjectKey> = vec![0];
    let milestone_keys: Vec<MilestoneKey> = vec![0, 1];

    ExtBuilder.build().execute_with(|| {
        deposit_initial_balance(&alice, &bob, additional_amount);
        create_project_multiple_milestones(alice, proposed_milestones);

        let project_index = 0;
        let milestone_index_1 = 0;
        let milestone_index_2 = 1;

        assert_ok!(<proposals::Pallet<Test>>::schedule_round(
            Origin::root(),
            System::block_number() - 1,
            System::block_number() + 1,
            project_keys
        ));

        let value = 100u64;
        assert_ok!(<proposals::Pallet<Test>>::contribute(
            Origin::signed(bob),
            project_index,
            CurrencyId::Native,
            value
        ));

        let mut milestone_index: Vec<MilestoneKey> = Vec::new();
        milestone_index.push(milestone_index_1);
        milestone_index.push(milestone_index_2);

        run_to_block(3);

        assert_ok!(Proposals::approve(
            Origin::root(),
            project_index,
            milestone_keys
        ));

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            milestone_index_1
        ));

        let voting_round_event_1 = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            voting_round_event_1,
            mock::Event::from(proposals::Event::VotingRoundCreated(1))
        );

        run_to_block(5);

        assert_ok!(Proposals::submit_milestone(
            Origin::signed(alice),
            project_index,
            milestone_index_2
        ));

        let voting_round_event_2 = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one EventRecord to be found")
            .event;
        assert_eq!(
            voting_round_event_2,
            mock::Event::from(proposals::Event::VotingRoundCreated(2))
        );
    });
}

//common helper methods
fn create_project(alice: AccountId) {
    assert_ok!(Proposals::create_project(
        Origin::signed(alice),
        //project name
        str::from_utf8(b"Farmer's Project Sudan")
            .unwrap()
            .as_bytes()
            .to_vec(),
        //project logo
        str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec(),
        //project description
        str::from_utf8(
            b"This project is aimed at providing decentralised funding for a farming project."
        )
        .unwrap()
        .as_bytes()
        .to_vec(),
        //website
        str::from_utf8(b"https://farmers.network")
            .unwrap()
            .as_bytes()
            .to_vec(),
        //milestone
        vec![ProposedMilestone {
            name: Vec::new(),
            percentage_to_unlock: 100
        }],
        //funds required
        1000000u64,
        CurrencyId::Native
    ));
}

fn create_project_multiple_milestones(
    alice: AccountId,
    proposed_milestones: Vec<ProposedMilestone>,
) {
    assert_ok!(Proposals::create_project(
        Origin::signed(alice),
        //project name
        str::from_utf8(b"Farmer's Project Sudan")
            .unwrap()
            .as_bytes()
            .to_vec(),
        //project logo
        str::from_utf8(b"Imbue Logo").unwrap().as_bytes().to_vec(),
        //project description
        str::from_utf8(
            b"This project is aimed at providing decentralised funding for a farming project."
        )
        .unwrap()
        .as_bytes()
        .to_vec(),
        //website
        str::from_utf8(b"https://farmers.network")
            .unwrap()
            .as_bytes()
            .to_vec(),
        //milestone
        proposed_milestones,
        //funds required
        1000000u64,
        CurrencyId::Native
    ));
}

fn deposit_initial_balance(alice: &AccountId, bob: &AccountId, additional_amount: u64) {
    let _ = Currencies::deposit(CurrencyId::Native, &alice, additional_amount);

    let _ = Currencies::deposit(CurrencyId::Native, &bob, additional_amount);
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            Proposals::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Proposals::on_initialize(System::block_number());
    }
}
