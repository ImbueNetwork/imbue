use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use pallet_disputes::DisputeResult;
use test_utils::*;

#[test]
fn you_can_actually_refund_after_dispute_success() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![BOB, CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let jury = vec![JURY_1, JURY_2];

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32
            ..milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();
        assert_ok!(Proposals::raise_dispute(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_keys.clone()
        ));
        let _ = complete_dispute::<Test>(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );
        // All milestones should be good for refund

        assert_ok!(Proposals::refund(RuntimeOrigin::signed(BOB), project_key));
    })
}

#[test]
fn refund_assert_milestone_state_change() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![BOB], 1_000_000u128);
        let milestones = get_milestones(10);
        let jury = vec![JURY_1, JURY_2];

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones,
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        // Only dispute some keys so that we can
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> =
            (0u32..5_u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_keys.clone()
        ));
        let _ = complete_dispute::<Test>(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );
        // All milestones should be good for refund

        assert_ok!(Proposals::refund(RuntimeOrigin::signed(BOB), project_key));
        let project_after_refund = Projects::<Test>::get(project_key).unwrap();
        assert_eq!(project_after_refund.refunded_funds, 500_000);
        for i in 0u32..10 {
            let milestone = project_after_refund.milestones.get(&i).unwrap();
            if i < 5 {
                assert!(milestone.can_refund);
                assert_eq!(
                    milestone.transfer_status,
                    Some(TransferStatus::Refunded {
                        on: frame_system::Pallet::<Test>::block_number()
                    })
                );
            } else {
                assert!(!milestone.can_refund);
                assert!(milestone.transfer_status.is_none());
            }
        }
    })
}

#[test]
fn refund_not_contributor() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![BOB, CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let jury = vec![JURY_1, JURY_2];

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32
            ..milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();
        assert_ok!(Proposals::raise_dispute(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_keys.clone()
        ));
        let _ = complete_dispute::<Test>(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );
        assert_noop!(
            Proposals::refund(RuntimeOrigin::signed(DAVE), project_key),
            Error::<Test>::OnlyContributorsCanInitiateRefund
        );
    })
}

#[test]
fn refund_deletes_project_when_all_funds_are_refunded() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![BOB, CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let jury = vec![JURY_1, JURY_2];

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32
            ..milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();
        assert_ok!(Proposals::raise_dispute(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_keys.clone()
        ));
        let _ = complete_dispute::<Test>(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );
        // All milestones should be good for refund

        Proposals::refund(RuntimeOrigin::signed(BOB), project_key).unwrap();
        assert!(!Projects::<Test>::contains_key(project_key));
    })
}

// The case where a project is in a dispute, and the dispute passes however, a milestone has also been approved and withdrawn
// before the refund has been called.
// Without the proper checks there will be a kind of double spend.
#[test]
fn withdraw_then_refund_no_double_spend() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![BOB], 1_000_000u128);
        let milestones = get_milestones(10);
        let milestone_key = 0;
        let alice_before_creation =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let bob_before_creation =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let jury = vec![JURY_1, JURY_2];

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32
            ..milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();
        let _ = Proposals::raise_dispute(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_keys.clone(),
        );
        let _ = complete_dispute::<Test>(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );
        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();

        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();
        // Milestone is approved, withdraw.
        assert_ok!(Proposals::withdraw(
            RuntimeOrigin::signed(ALICE),
            project_key
        ));
        let project_after_withdraw = Projects::<Test>::get(project_key).unwrap();
        let alice_after_withdraw =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        // Assert that alice has recieved the withdraw.
        assert!(alice_after_withdraw > alice_before_creation);
        let refund_fee = <Test as Config>::ImbueFee::get().mul_floor(
            project_after_withdraw.raised_funds - project_after_withdraw.withdrawn_funds,
        );
        // Leaves us with 9 milestones left which we will refund.
        assert_ok!(Proposals::refund(RuntimeOrigin::signed(BOB), project_key));
        let bob_after_refund =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        assert_eq!(
            bob_after_refund,
            (bob_before_creation - project_after_withdraw.withdrawn_funds - refund_fee),
            "bobs shizzle aint what it should be."
        );
    })
}

// The reverse case of withdraw_then_refund_no_double_spend
// essentially if a milestone is refunded one cannot withdraw an approved milestone as its already gone.
#[test]
fn refund_then_withdraw_no_double_spend() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![BOB], 1_000_000u128);
        let milestones = get_milestones(10);
        let milestone_key = 0;
        let _alice_before_creation =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let _bob_before_creation =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &ALICE);
        let jury = vec![JURY_1, JURY_2];

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones,
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> =
            (0u32..5_u32).collect::<Vec<u32>>().try_into().unwrap();
        let _ = Proposals::raise_dispute(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_keys.clone(),
        );
        let _ = complete_dispute::<Test>(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );
        let _ =
            Proposals::submit_milestone(RuntimeOrigin::signed(ALICE), project_key, milestone_key)
                .unwrap();

        let _ = Proposals::vote_on_milestone(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_key,
            true,
        )
        .unwrap();
        assert_ok!(Proposals::refund(RuntimeOrigin::signed(BOB), project_key));
        assert_noop!(
            Proposals::withdraw(RuntimeOrigin::signed(ALICE), project_key),
            Error::<Test>::NoAvailableFundsToWithdraw
        );
    })
}

#[test]
fn refund_check_refund_amount() {
    build_test_externality().execute_with(|| {
        let bob_pre_creation =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        let charlie_pre_creation =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &CHARLIE);
        let per_contribution = 100000u128;
        let contributions = get_contributions::<Test>(vec![BOB, CHARLIE], per_contribution as u128);
        let milestones = get_milestones(10);
        let jury = vec![JURY_1, JURY_2];

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32
            ..milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();
        assert_ok!(Proposals::raise_dispute(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_keys.clone()
        ));
        let _ = complete_dispute::<Test>(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );
        // All milestones should be good for refund

        assert_ok!(Proposals::refund(RuntimeOrigin::signed(BOB), project_key));
        let bob_post_refund =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        let charlie_post_refund =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &CHARLIE);
        let per_fee = <Test as Config>::ImbueFee::get().mul_floor(per_contribution);
        assert_eq!(
            bob_pre_creation - per_fee,
            bob_post_refund,
            "bobo didnt get his money back!!"
        );
        assert_eq!(
            charlie_pre_creation - per_fee,
            charlie_post_refund,
            "charlie didnt get his money back!!"
        );
    })
}

#[test]
fn refund_takes_imbue_fee() {
    build_test_externality().execute_with(|| {
        let bob_pre_creation =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        let charlie_pre_creation =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &CHARLIE);
        let fee_account_pre_creation = <Test as Config>::MultiCurrency::free_balance(
            CurrencyId::Native,
            &<Test as Config>::ImbueFeeAccount::get(),
        );
        let per_contribution = 500000u128;

        let contributions = get_contributions::<Test>(vec![BOB, CHARLIE], per_contribution as u128);
        let milestones = get_milestones(10);
        let jury = vec![JURY_1, JURY_2];

        let project_key = create_and_fund_project::<Test>(
            ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
            jury,
        )
        .unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32
            ..milestones.len() as u32)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();
        assert_ok!(Proposals::raise_dispute(
            RuntimeOrigin::signed(BOB),
            project_key,
            milestone_keys.clone()
        ));
        let _ = complete_dispute::<Test>(
            project_key,
            milestone_keys.into_inner(),
            DisputeResult::Success,
        );
        // All milestones should be good for refund

        assert_ok!(Proposals::refund(RuntimeOrigin::signed(BOB), project_key));

        let bob_post_refund =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        let charlie_post_refund =
            <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &CHARLIE);
        let fee_account_post_creation = <Test as Config>::MultiCurrency::free_balance(
            CurrencyId::Native,
            &<Test as Config>::ImbueFeeAccount::get(),
        );
        let per_fee = <Test as Config>::ImbueFee::get().mul_floor(per_contribution);

        // Assert that the fee has been taken from each and transferred to ImbueFeeAccount.
        assert_eq!(
            bob_pre_creation - bob_post_refund,
            per_fee,
            "bobs fee hasnt been taken out correctly."
        );
        assert_eq!(
            charlie_pre_creation - charlie_post_refund,
            per_fee,
            "charlies fee hasnt been taken out correctly."
        );
        assert_eq!(
            fee_account_post_creation - fee_account_pre_creation,
            per_fee * 2,
            "total fee hasnt added correctly."
        );
    })
}
