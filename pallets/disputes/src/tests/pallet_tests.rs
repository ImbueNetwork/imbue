use super::test_utils::*;

#[test]
fn raise_dispute_assert_state() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        let expiration_block =
            <Test as Config>::VotingTimeLimit::get() + frame_system::Pallet::<Test>::block_number();
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        assert!(PalletDisputes::disputes(dispute_key).is_some());
        assert_eq!(1, PalletDisputes::disputes(dispute_key).iter().count());
        assert!(DisputesFinaliseOn::<Test>::get(expiration_block).contains(&dispute_key));
    });
}

#[test]
fn raise_dispute_assert_event() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        System::assert_last_event(RuntimeEvent::PalletDisputes(Event::<Test>::DisputeRaised {
            dispute_key: dispute_key,
        }));
    });
}

///testing when trying to insert more than max number of disputes allowed in a block
#[test]
fn raise_dispute_assert_event_too_many_disputes() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let disputes_limit = <Test as Config>::MaxDisputesPerBlock::get();
        (0..=disputes_limit).for_each(|i| {
            let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
            let specifics = get_specifics::<Test>(vec![0, 1]);
            if i != disputes_limit {
                assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
                    i, *ALICE, jury, specifics,
                ));
                System::assert_last_event(RuntimeEvent::PalletDisputes(
                    Event::<Test>::DisputeRaised { dispute_key: i },
                ));
            } else {
                let actual_result = <PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
                    i, *ALICE, jury, specifics,
                );
                assert_noop!(actual_result, Error::<Test>::TooManyDisputesThisBlock);
            }
        });
    });
}

#[test]
fn raise_dispute_already_exists() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury.clone(),
            specifics.clone(),
        ));
        assert_noop!(
            <PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
                dispute_key,
                *ALICE,
                jury,
                specifics
            ),
            Error::<Test>::DisputeAlreadyExists
        );
    });
}

#[test]
fn vote_on_dispute_assert_state() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        let dispute_before_vote = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert_eq!(0, dispute_before_vote.votes.len());
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key,
            true
        ));
        let dispute_after_vote = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        let vote = dispute_after_vote.votes.get(&BOB).unwrap();
        assert_eq!(true, *vote);
        assert_eq!(1, dispute_after_vote.votes.len());
    });
}

#[test]
fn vote_on_dispute_assert_last_event() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));

        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key,
            true
        ));
        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeVotedOn { who: *BOB },
        ));
    });
}

#[test]
fn vote_on_dispute_assert_last_event_on_initialize() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key,
            true
        ));
        ///trying to expire the timelimit for the given dispute
        let current_block = frame_system::Pallet::<Test>::current_block_number();
        run_to_block::<Test>(current_block + <Test as Config>::VotingTimeLimit::get());
        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeCompleted {
                dispute_key,
                dispute_result: DisputeResult::Success,
            },
        ));
    });
}

#[test]
fn vote_on_dispute_autofinalises_on_unanimous_yes() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key,
            true
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*CHARLIE),
            dispute_key,
            true
        ));
        //verify that the dispute has been removed once auto_finalization is done in case of unanimous yes
        assert_eq!(0, PalletDisputes::disputes(dispute_key).iter().count());
    });
}

#[test]
fn vote_on_dispute_autofinalises_on_unanimous_no() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key,
            false
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*CHARLIE),
            dispute_key,
            false
        ));
        //verify that the dispute has been removed once auto_finalization is done in case of unanimous no
        assert_eq!(0, PalletDisputes::disputes(dispute_key).iter().count());
    });
}

///SHANKAR: What does this mean?
#[test]
fn try_auto_finalise_removes_autofinalise() {
    new_test_ext().execute_with(|| {
        new_test_ext().execute_with(|| {
            let dispute_key = 10;
            let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
            let specifics = get_specifics::<Test>(vec![0, 1]);
            assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
                dispute_key,
                *ALICE,
                jury,
                specifics,
            ));
            assert_ok!(PalletDisputes::vote_on_dispute(
                RuntimeOrigin::signed(*BOB),
                dispute_key,
                false
            ));
            assert_ok!(PalletDisputes::vote_on_dispute(
                RuntimeOrigin::signed(*CHARLIE),
                dispute_key,
                false
            ));
            //verify that the dispute has been removed once auto_finalization is done in case of unanimous no
            assert_eq!(0, PalletDisputes::disputes(dispute_key).iter().count());
            //After the dispute has been autofinalized and the we again tru to autofinalize it throws an error saying that
            // the dispute doesnt exists as it has been removed
            assert_noop!(
                Dispute::<Test>::try_finalise_with_result(dispute_key, DisputeResult::Success),
                Error::<Test>::DisputeDoesNotExist
            );
        });
    });
}

///testing if the non jury account tries to vote it should throw the error saying its not a jury account
#[test]
fn vote_on_dispute_not_jury_account() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids = get_specifics::<Test>(vec![0]);

        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specific_ids
        ));
        assert_noop!(
            PalletDisputes::vote_on_dispute(RuntimeOrigin::signed(*CHARLIE), dispute_key, true),
            Error::<Test>::NotAJuryAccount
        );
    });
}

///trying to vote on a dispute that doesn't exists which result in the error throwing dispute does not exists
#[test]
fn vote_on_dispute_dispute_doesnt_exist() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids = get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specific_ids
        ));

        assert_noop!(
            PalletDisputes::vote_on_dispute(RuntimeOrigin::signed(*BOB), 1, true),
            Error::<Test>::DisputeDoesNotExist
        );
    });
}

///trying to extend the voting time  on a dispute that doesn't exists which result in the error throwing dispute does not exists
#[test]
fn extend_dispute_dispute_doesnt_exist() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids = get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specific_ids
        ));
        assert_noop!(
            PalletDisputes::extend_dispute(RuntimeOrigin::signed(*BOB), 1),
            Error::<Test>::DisputeDoesNotExist
        );
    });
}

///testing to extend the time for voting from a not jury account, it should throw the error saying its not a jury account
#[test]
fn extend_dispute_not_a_jury_account() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids = get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specific_ids
        ));
        assert_noop!(
            PalletDisputes::extend_dispute(RuntimeOrigin::signed(*CHARLIE), dispute_key),
            Error::<Test>::NotAJuryAccount
        );
    });
}

/// testing trying to extend the voting on a dispute which has already been extended and should throw Dispute Already Extended error
#[test]
fn extend_dispute_already_extended() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids = get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specific_ids
        ));
        assert_ok!(PalletDisputes::extend_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key
        ));
        assert_noop!(
            PalletDisputes::extend_dispute(RuntimeOrigin::signed(*BOB), dispute_key),
            Error::<Test>::DisputeAlreadyExtended
        );
    });
}

/// testing trying to extend the voting time and it successfully extend by setting the flag to true
#[test]
fn extend_dispute_works_assert_last_event() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids = get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specific_ids
        ));
        let d = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert!(!d.is_extended);
        assert_ok!(PalletDisputes::extend_dispute(
            RuntimeOrigin::signed(*BOB),
            10
        ));
        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeExtended {
                dispute_key: dispute_key,
            },
        ));
    });
}

#[test]
fn extend_dispute_works_assert_state() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids = get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specific_ids
        ));
        let d = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert!(!d.is_extended);
        assert_eq!(11, d.expiration);
        assert_ok!(PalletDisputes::extend_dispute(
            RuntimeOrigin::signed(*BOB),
            10
        ));
        let d = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert!(d.is_extended);
        assert_eq!(21, d.expiration);
    });
}

#[test]
fn extend_dispute_too_many_disputes() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let disputes_limit = <Test as Config>::MaxDisputesPerBlock::get();
        (0u32..=disputes_limit).for_each(|i| {
            let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
            let specifics = get_specifics::<Test>(vec![0, 1]);
            {
                let dispute = Dispute {
                    raised_by: *ALICE,
                    votes: BoundedBTreeMap::default(),
                    jury: jury,
                    specifiers: Default::default(),
                    is_extended: false,
                    expiration: 27,
                };
                Disputes::<Test>::insert(i, dispute);
                let actual_result = PalletDisputes::extend_dispute(RuntimeOrigin::signed(*BOB), i);
                if i == disputes_limit {
                    assert_noop!(actual_result, Error::<Test>::TooManyDisputesThisBlock);
                }
            };
        });
    });
}

#[test]
fn calculate_winner_works_dispute_success() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB, *FERDIE]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key,
            true
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*CHARLIE),
            dispute_key,
            false
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*FERDIE),
            dispute_key,
            true
        ));
        let current_block = frame_system::Pallet::<Test>::current_block_number();
        run_to_block::<Test>(current_block + <Test as Config>::VotingTimeLimit::get());
        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeCompleted {
                dispute_key,
                dispute_result: DisputeResult::Success,
            },
        ));
    });
}

#[test]
fn calculate_winner_works_dispute_failure() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB, *FERDIE]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key,
            true
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*CHARLIE),
            dispute_key,
            false
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*FERDIE),
            dispute_key,
            false
        ));
        let current_block = frame_system::Pallet::<Test>::current_block_number();
        run_to_block::<Test>(current_block + <Test as Config>::VotingTimeLimit::get());
        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeCompleted {
                dispute_key,
                dispute_result: DisputeResult::Failure,
            },
        ));
    });
}

///e2e
#[test]
fn e2e() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*BOB),
            dispute_key,
            true
        ));
        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*CHARLIE),
            dispute_key,
            true
        ));
        //verify that the dispute has been removed once auto_finalization is done in case of unanimous yes
        assert_eq!(0, PalletDisputes::disputes(dispute_key).iter().count());
    });
}
