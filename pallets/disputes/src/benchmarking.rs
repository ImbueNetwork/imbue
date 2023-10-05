#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::traits::DisputeRaiser;
use crate::Pallet as PalletDisputes;
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, BoundedVec};
use orml_traits::MultiCurrency;
use frame_system::Pallet as System;
use sp_runtime::SaturatedConversion;
use sp_std::vec::Vec;

#[benchmarks( where <T as frame_system::Config>::AccountId: AsRef<[u8]>, Event::<T>: Into<<T as frame_system::Config>::RuntimeEvent>)]
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
            <Pallet<T> as DisputeRaiser<<T as frame_system::Config>::AccountId>>::raise_dispute(
                10u32.into(),
                alice,
                jury,
                specifics,
            );
        }
    }

    #[benchmark]
    fn extend_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![bob.clone()]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);

        <Pallet<T> as DisputeRaiser<<T as frame_system::Config>::AccountId>>::raise_dispute(
            10u32.into(),
            alice.clone(),
            jury,
            specifics,
        );

        #[extrinsic_call]
        <Pallet<T>>::extend_dispute(RawOrigin::Signed(bob), 10u32.into());
    }

    #[benchmark]
    fn vote_on_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![bob.clone()]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);

        <Pallet<T> as DisputeRaiser<<T as frame_system::Config>::AccountId>>::raise_dispute(
            10u32.into(),
            alice.clone(),
            jury,
            specifics,
        );

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
        <Pallet<T> as DisputeRaiser<<T as frame_system::Config>::AccountId>>::raise_dispute(
            dispute_key,
            alice.clone(),
            jury,
            specifics,
        );

        #[extrinsic_call]
        <Pallet<T>>::force_fail_dispute(RawOrigin::Root, dispute_key);

        System::<T>::assert_last_event(
            Event::<T>::DisputeCompleted{
                dispute_key,
                dispute_result: DisputeResult::Failure,
            }.into()
        );
    }

    #[benchmark]
    fn force_succeed_dispute() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let jury = get_jury::<T>(vec![bob.clone()]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);
        let dispute_key = 10u32.into();
        <Pallet<T> as DisputeRaiser<<T as frame_system::Config>::AccountId>>::raise_dispute(
            dispute_key,
            alice.clone(),
            jury,
            specifics,
        );

        #[extrinsic_call]
        <Pallet<T>>::force_succeed_dispute(RawOrigin::Root, dispute_key);

        System::<T>::assert_last_event(
            Event::<T>::DisputeCompleted{
                dispute_key,
                dispute_result: DisputeResult::Success,
            }.into()
        );
    }

    #[benchmark]
    fn calculate_winner() {
        let alice: AccountIdOf<T> = account("ALICE", 0, 0);
        let bob: AccountIdOf<T> = account("BOB", 0, 0);
        let charlie: AccountIdOf<T> = account("CHARLIE", 0, 0);
        let jury = get_jury::<T>(vec![bob.clone(),charlie.clone()]);
        let specifics = get_specifics::<T>(vec![0u32.into(), 1u32.into()]);
        let dispute_key = 10u32.into();
        assert_ok!(Dispute::<T>::new(10u32.into(), alice, jury, specifics));
        let mut dispute = Disputes::<T>::get(dispute_key).expect("just inserted, should exist.");
        assert_ok!(dispute.try_add_vote(charlie, true, dispute_key));
        assert_ok!(dispute.try_add_vote(bob, true, dispute_key));

        #[block]{
            assert_eq!(dispute.calculate_winner(), DisputeResult::Success);
        }
    }

    impl_benchmark_test_suite!(
        PalletDisputes,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}

pub fn get_jury<T: Config>(
    accounts: Vec<<T as frame_system::Config>::AccountId>,
) -> BoundedVec<AccountIdOf<T>, <T as Config>::MaxJurySize> {
    accounts.try_into().expect("too many jury members")
}

pub fn get_specifics<T: Config>(
    specifics: Vec<T::SpecificId>,
) -> BoundedVec<T::SpecificId, T::MaxSpecifics> {
    specifics.try_into().expect("too many specific ids.")
}
