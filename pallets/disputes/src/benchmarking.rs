#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::traits::DisputeRaiser;
use crate::Pallet as PalletDisputes;
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::Get, BoundedVec};
use frame_system::Pallet as System;
use sp_std::{vec, vec::Vec};

#[benchmarks(where Event::<T>: Into<<T as frame_system::Config>::RuntimeEvent>)]
mod benchmarks {
    use super::*;
    use frame_support::dispatch::RawOrigin;

    #[benchmark]
    fn raise_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![alice.clone(), bob]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);
        #[block]
        {
            <Pallet<T> as DisputeRaiser<AccountIdOf<T>>>::raise_dispute(
                10u32.into(),
                alice,
                jury,
                specifics,
            )
            .unwrap();
        }
    }

    #[benchmark]
    fn extend_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![bob.clone()]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);

        assert_ok!(<Pallet<T> as DisputeRaiser<AccountIdOf<T>>>::raise_dispute(
            10u32.into(),
            alice.clone(),
            jury,
            specifics,
        ));

        #[extrinsic_call]
        <Pallet<T>>::extend_dispute(RawOrigin::Signed(bob), 10u32.into());
    }

    // Worst case atm is causing it to autofinalise.
    #[benchmark]
    fn vote_on_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![bob.clone()]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);

        assert_ok!(<Pallet<T> as DisputeRaiser<AccountIdOf<T>>>::raise_dispute(
            10u32.into(),
            alice.clone(),
            jury,
            specifics,
        ));

        #[extrinsic_call]
        <Pallet<T>>::vote_on_dispute(RawOrigin::Signed(bob), 10u32.into(), true);
    }

    #[benchmark]
    fn force_fail_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![bob.clone()]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);
        let dispute_key = 10u32.into();
        assert_ok!(<Pallet<T> as DisputeRaiser<AccountIdOf<T>>>::raise_dispute(
            dispute_key,
            alice.clone(),
            jury,
            specifics,
        ));

        #[extrinsic_call]
        <Pallet<T>>::force_fail_dispute(RawOrigin::Root, dispute_key);

        System::<T>::assert_last_event(
            Event::<T>::DisputeCompleted {
                dispute_key,
                dispute_result: DisputeResult::Failure,
            }
            .into(),
        );
    }

    #[benchmark]
    fn force_succeed_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![bob.clone()]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);
        let dispute_key = 10u32.into();
        assert_ok!(<Pallet<T> as DisputeRaiser<AccountIdOf<T>>>::raise_dispute(
            dispute_key,
            alice.clone(),
            jury,
            specifics,
        ));

        #[extrinsic_call]
        <Pallet<T>>::force_succeed_dispute(RawOrigin::Root, dispute_key);

        System::<T>::assert_last_event(
            Event::<T>::DisputeCompleted {
                dispute_key,
                dispute_result: DisputeResult::Success,
            }
            .into(),
        );
    }

    // Linear relationship with jury members.
    #[benchmark]
    fn calculate_winner() {
        let dispute_key = 10u32.into();
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);

        let mut accounts: Vec<AccountIdOf<T>> = Vec::new();
        for i in 0..T::MaxJurySize::get() {
            let acc: AccountIdOf<T> = account("ANY", i, 0);
            accounts.push(acc)
        }
        let jury = get_jury::<T>(accounts.clone());

        assert_ok!(Dispute::<T>::new(
            10u32.into(),
            alice,
            jury.clone(),
            specifics
        ));
        let mut dispute = Disputes::<T>::get(dispute_key).expect("just inserted, should exist.");

        for i in 0..T::MaxJurySize::get() {
            let acc = jury[i as usize].clone();
            assert_ok!(dispute.try_add_vote(acc, true, dispute_key));
        }

        #[block]
        {
            dispute.calculate_winner();
        }
    }

    impl_benchmark_test_suite!(
        PalletDisputes,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}

pub fn get_jury<T: Config>(
    accounts: Vec<AccountIdOf<T>>,
) -> BoundedVec<AccountIdOf<T>, <T as Config>::MaxJurySize> {
    accounts.try_into().expect("too many jury members")
}

pub fn get_specifics<T: Config>(
    specifics: Vec<T::SpecificId>,
) -> BoundedVec<T::SpecificId, T::MaxSpecifics> {
    specifics.try_into().expect("too many specific ids.")
}
