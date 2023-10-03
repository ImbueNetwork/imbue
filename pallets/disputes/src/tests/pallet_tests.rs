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
        assert_eq!(dispute_after_vote.votes.get(&BOB).unwrap(), &true);
        assert_eq!(1, dispute_after_vote.votes.len());

        assert_ok!(PalletDisputes::vote_on_dispute(
            RuntimeOrigin::signed(*CHARLIE),
            dispute_key,
            false
        ));

        let dispute_after_vote = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert_eq!(dispute_after_vote.votes.get(&CHARLIE).unwrap(), &false);
        assert_eq!(dispute_after_vote.votes.get(&BOB).unwrap(), &true);
        assert_eq!(2, dispute_after_vote.votes.len());
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
            Event::<Test>::DisputeVotedOn {
                dispute_key,
                who: *BOB,
                vote: true,
            },
        ));
    });
}

#[test]
fn on_initialize_tries_to_finalise_assert_event() {
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
        // Trying to expire the timelimit for the given dispute
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
        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeCompleted {
                dispute_key,
                dispute_result: DisputeResult::Success,
            },
        ));
        //verify that the dispute has been removed once auto_finalization is done in case of unanimous yes
        assert!(Disputes::<Test>::get(dispute_key).is_none());
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
        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeCompleted {
                dispute_key,
                dispute_result: DisputeResult::Failure,
            },
        ));
        //verify that the dispute has been removed once auto_finalization is done in case of unanimous no
        assert!(Disputes::<Test>::get(dispute_key).is_none());
    });
}

// Ensure that when a dispute is finalised that the dispute is not given to the on_initialise
// to try and auto finalise again.
#[test]
fn try_auto_finalise_removes_autofinalise_storage() {
    new_test_ext().execute_with(|| {
        new_test_ext().execute_with(|| {
            let dispute_key_1 = 10;
            let dispute_key_2 = 11;
            let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
            let specifics = get_specifics::<Test>(vec![0, 1]);
            let expiry_block = frame_system::Pallet::<Test>::block_number()
                + <Test as Config>::VotingTimeLimit::get();
            assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
                dispute_key_1,
                *ALICE,
                jury.clone(),
                specifics.clone(),
            ));
            assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
                dispute_key_2,
                *ALICE,
                jury,
                specifics,
            ));
            assert_ok!(PalletDisputes::vote_on_dispute(
                RuntimeOrigin::signed(*BOB),
                dispute_key_1,
                false
            ));
            assert_ok!(PalletDisputes::vote_on_dispute(
                RuntimeOrigin::signed(*BOB),
                dispute_key_2,
                true
            ));
            assert_ok!(Dispute::<Test>::try_finalise_with_result(
                dispute_key_1,
                DisputeResult::Success
            ));
            let finalising_disputes = DisputesFinaliseOn::<Test>::get(expiry_block);

            assert!(finalising_disputes.contains(&dispute_key_2));
            assert!(!finalising_disputes.contains(&dispute_key_1));
            assert_eq!(finalising_disputes.len(), 1);
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
            dispute_key
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
        let initial_expiry =
            frame_system::Pallet::<Test>::block_number() + <Test as Config>::VotingTimeLimit::get();
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specific_ids
        ));
        // Assert state before extension.
        let d = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert!(!d.is_extended);
        assert_eq!(initial_expiry, d.expiration);

        // Assert it will autofinalise on the old expiry block
        let autofinalising = DisputesFinaliseOn::<Test>::get(d.expiration);
        assert!(autofinalising.len() == 1);
        assert!(autofinalising.contains(&dispute_key));

        assert_ok!(PalletDisputes::extend_dispute(
            RuntimeOrigin::signed(*BOB),
            10
        ));

        // Assert state after extension.
        let d = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert!(d.is_extended);
        assert_eq!(
            initial_expiry + <Test as Config>::VotingTimeLimit::get(),
            d.expiration
        );
        // Assert it wont autofinalise on the old expiry block
        let autofinalising = DisputesFinaliseOn::<Test>::get(initial_expiry);
        assert!(autofinalising.len() == 0);

        // Assert it will on the new exipry.
        let autofinalising = DisputesFinaliseOn::<Test>::get(d.expiration);
        assert!(autofinalising.len() == 1);
        assert!(autofinalising.contains(&dispute_key));
    });
}

// Where a dispute has been raised in the past and then extended on top of MaxDisputesPerBlock.
#[test]
fn extend_dispute_too_many_disputes() {
    new_test_ext().execute_with(|| {
        let disputes_limit = <Test as Config>::MaxDisputesPerBlock::get();
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);

        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            u32::MAX,
            *ALICE,
            jury.clone(),
            specifics.clone(),
        ));

        run_to_block::<Test>(
            frame_system::Pallet::<Test>::block_number() + <Test as Config>::VotingTimeLimit::get(),
        );
        (0u32..disputes_limit).for_each(|i| {
            assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
                i,
                *ALICE,
                jury.clone(),
                specifics.clone(),
            ));
        });
        // It will never exists here unless the VotingTimeLimit is changed to a value smaller than before.
        assert_noop!(
            PalletDisputes::extend_dispute(RuntimeOrigin::signed(*BOB), u32::MAX),
            Error::<Test>::DisputeDoesNotExist
        );
    });
}

#[test]
fn try_auto_finalise_without_votes_fails() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*CHARLIE, *BOB]);
        let specifics = get_specifics::<Test>(vec![0, 1]);
        let expiry_block =
            frame_system::Pallet::<Test>::block_number() + <Test as Config>::VotingTimeLimit::get();
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(
            dispute_key,
            *ALICE,
            jury,
            specifics,
        ));
        // Noone votes, go directly to expiry block
        run_to_block::<Test>(expiry_block);

        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeCompleted {
                dispute_key: dispute_key,
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
