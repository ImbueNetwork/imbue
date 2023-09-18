use crate::traits::*;
use crate::{mock::*, pallet::*, pallet};
use frame_support::traits::Len;
use frame_support::{assert_noop, assert_ok, traits::Hooks};
use sp_arithmetic::traits::One;
use sp_runtime::{BoundedVec, Saturating};
use test_utils::*;

mod test_utils {
    use super::*;
    pub fn run_to_block<T: Config>(n: T::BlockNumber)
    where
        T::BlockNumber: Into<u64>,
    {
        loop {
            let mut block: T::BlockNumber = frame_system::Pallet::<T>::block_number();
            if block >= n {
                break;
            }
            block = block.saturating_add(<T::BlockNumber as One>::one());
            frame_system::Pallet::<T>::set_block_number(block);
            frame_system::Pallet::<T>::on_initialize(block);
            PalletDisputes::on_initialize(block.into());
        }
    }

    pub fn get_jury<T: Config>(accounts: Vec<AccountIdOf<T>>) -> BoundedVec<AccountIdOf<T>, <T as Config>::MaxJurySize> {
        accounts.try_into().expect("too many jury members")
    }

    pub fn get_specifics<T: Config>(specifics: Vec<T::SpecificId>) -> BoundedVec<T::SpecificId, T::MaxSpecifics> {
        specifics.try_into().expect("too many specific ids.")
    }
}

#[test]
fn raise_dispute_assert_state() {
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
        assert!(PalletDisputes::disputes(dispute_key).is_some());
        assert_eq!(1, PalletDisputes::disputes(dispute_key).iter().count());
    });
}

#[test]
fn raise_dispute_assert_event() {
    new_test_ext().execute_with(|| {
        todo!()
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
            <PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(dispute_key, *ALICE, jury, specifics),
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
        assert_ok!(PalletDisputes::vote_on_dispute(RuntimeOrigin::signed(*BOB), dispute_key, true));
        todo!()
    });
}


// TODO Working on this upon the approval of the finalization
// FELIX: shankar what does this mean? ^^
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

        assert_ok!(PalletDisputes::vote_on_dispute(RuntimeOrigin::signed(*BOB), dispute_key, true));
        System::assert_last_event(RuntimeEvent::PalletDisputes(
            Event::<Test>::DisputeVotedOn { who: *BOB },
        ));
    });
}

///testing if the non jury account tries to vote it should throw the error saying its not a jury account
#[test]
fn vote_on_dispute_not_jury_account() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids =get_specifics::<Test>(vec![0]);

        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(dispute_key, *ALICE, jury, specific_ids));
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
        let specific_ids =get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(dispute_key, *ALICE, jury, specific_ids));

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
        let specific_ids =get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(dispute_key, *ALICE, jury, specific_ids));
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
        let specific_ids =get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(dispute_key, *ALICE, jury, specific_ids));
        assert_noop!(
            PalletDisputes::extend_dispute(RuntimeOrigin::signed(*CHARLIE), dispute_key),
            Error::<Test>::NotAJuryAccount
        );
    });
}

/// testing trying to extend the voting on a dispute which has already been extended and should throw Dispute Already Extended error
#[test]
fn test_extending_the_voting_which_has_already_been_extended() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids =get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(dispute_key, *ALICE, jury, specific_ids));
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
fn test_successfully_extending_the_voting_on_a_dispute() {
    new_test_ext().execute_with(|| {
        let dispute_key = 10;
        let jury = get_jury::<Test>(vec![*BOB]);
        let specific_ids =get_specifics::<Test>(vec![0]);
        assert_ok!(<PalletDisputes as DisputeRaiser<AccountId>>::raise_dispute(dispute_key, *ALICE, jury, specific_ids));
        let d = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert!(!d.is_extended);
        assert_ok!(PalletDisputes::extend_dispute(
            RuntimeOrigin::signed(*BOB),
            10
        ));
        let d = Disputes::<Test>::get(dispute_key).expect("dispute should exist");
        assert!(d.is_extended);
    });
}
