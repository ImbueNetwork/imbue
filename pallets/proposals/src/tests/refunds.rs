use frame_support::{assert_noop, assert_ok};
use crate::{mock::*, *};
use test_utils::*;
use pallet_disputes::DisputeResult;

#[test]
fn you_can_actually_refund_after_dispute_success() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);
        // All milestones should be good for refund
        
        assert_ok!(Proposals::refund(RuntimeOrigin::signed(*BOB), project_key));
    })
}


#[test]
fn refund_not_contributor() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);
        assert_noop!(Proposals::refund(RuntimeOrigin::signed(*DAVE), project_key), Error::<Test>::OnlyContributorsCanInitiateRefund);
    })
}

#[test]
fn refund_deletes_project_when_all_funds_are_refunded() {
    build_test_externality().execute_with(|| {
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], 1_000_000u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);
        // All milestones should be good for refund
        
        let _ = Proposals::refund(RuntimeOrigin::signed(*BOB), project_key).unwrap();
        assert!(!Projects::<Test>::contains_key(project_key));
    })
}

// The case where a project is in a dispute, and the dispute passes however, a milestone has also been approved
// before the refund has been called.
// Without the proper checks there will be a kind of double spend.
#[test]
fn refund_only_transfers_milestones_which_havent_been_withdrawn() {
    build_test_externality().execute_with(|| {

    })
}

#[test]
fn refund_check_refund_amount() {
    build_test_externality().execute_with(|| {
        let bob_pre_creation = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        let charlie_pre_creation = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &CHARLIE);
        let per_contribution = 100000u64;
        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], per_contribution as u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);
        // All milestones should be good for refund
        
        assert_ok!(Proposals::refund(RuntimeOrigin::signed(*BOB), project_key));
        let bob_post_refund = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        let charlie_post_refund = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &CHARLIE);
        let per_fee = <Test as Config>::ImbueFee::get().mul_floor(per_contribution);
        assert_eq!(bob_pre_creation - per_fee, bob_post_refund , "bobo didnt get his money back!!");
        assert_eq!(charlie_pre_creation - per_fee, charlie_post_refund , "charlie didnt get his money back!!");
    })
}

#[test]
fn refund_takes_imbue_fee() {
    build_test_externality().execute_with(|| {
        let bob_pre_creation = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        let charlie_pre_creation = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &CHARLIE);
        let fee_account_pre_creation = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &<Test as Config>::ImbueFeeAccount::get());
        let per_contribution = 500000u64;

        let contributions = get_contributions::<Test>(vec![*BOB, *CHARLIE], per_contribution as u128);
        let milestones = get_milestones(10);
        let project_key = create_and_fund_project::<Test>(
            *ALICE,
            contributions,
            milestones.clone(),
            CurrencyId::Native,
        ).unwrap();
        let milestone_keys: BoundedVec<u32, <Test as Config>::MaxMilestonesPerProject> = (0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().unwrap();
        assert_ok!(Proposals::raise_dispute(RuntimeOrigin::signed(*BOB), project_key, milestone_keys.clone()));
        let _ = complete_dispute::<Test>(project_key, milestone_keys.into_inner(), DisputeResult::Success);
        // All milestones should be good for refund
        
        assert_ok!(Proposals::refund(RuntimeOrigin::signed(*BOB), project_key));

        let bob_post_refund = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &BOB);
        let charlie_post_refund = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &CHARLIE);
        let fee_account_post_creation = <Test as Config>::MultiCurrency::free_balance(CurrencyId::Native, &<Test as Config>::ImbueFeeAccount::get());
        let per_fee = <Test as Config>::ImbueFee::get().mul_floor(per_contribution);

        // Assert that the fee has been taken from each and transferred to ImbueFeeAccount.
        assert_eq!(bob_pre_creation - bob_post_refund, per_fee, "bobs fee hasnt been taken out correctly.");
        assert_eq!(charlie_pre_creation - charlie_post_refund, per_fee, "charlies fee hasnt been taken out correctly.");
        assert_eq!(fee_account_post_creation - fee_account_pre_creation, per_fee * 2, "total fee hasnt added correctly.");
    })
}