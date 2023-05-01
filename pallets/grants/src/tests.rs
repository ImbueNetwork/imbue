use crate::mock::*;
use crate::pallet::{
    BoundedApprovers, BoundedPMilestones, Config, Error, GrantId, PendingGrants,
    ProposedMilestoneWithInfo,
};
use common_types::{CurrencyId, TreasuryOrigin};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use sp_runtime::DispatchError::BadOrigin;

#[test]
fn ensure_milestone_percent_equal_100() {
    new_test_ext().execute_with(|| {
        let milestones: BoundedPMilestones<Test> = vec![ProposedMilestoneWithInfo {
            percent: 50u8,
            ipfs_hash: Default::default(),
        }]
        .try_into()
        .expect("qed");

        assert_noop!(
            Grant::submit_initial_grant(
                RuntimeOrigin::signed(*ALICE),
                Default::default(),
                milestones,
                get_approvers(5),
                CurrencyId::Native,
                10_000u32.into(),
                TreasuryOrigin::Kusama,
                Default::default()
            ),
            Error::<Test>::MustSumTo100
        );
    });
}

#[test]
fn create_grant_already_exists() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let milestones2 = get_milestones(10);
        let approvers = get_approvers(10);
        let approvers2 = get_approvers(10);
        let grant_id: H256 = Default::default();

        let _ = Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            milestones,
            approvers,
            CurrencyId::Native,
            10_000u32.into(),
            TreasuryOrigin::Kusama,
            grant_id,
        );
        assert_noop!(
            Grant::submit_initial_grant(
                RuntimeOrigin::signed(*ALICE),
                Default::default(),
                milestones2,
                approvers2,
                CurrencyId::Native,
                10_000u32.into(),
                TreasuryOrigin::Kusama,
                grant_id
            ),
            Error::<Test>::GrantAlreadyExists
        );
    });
}

#[test]
fn edit_grant_only_submitter_can_edit() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let approvers = get_approvers(10);
        let _ = Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            milestones,
            approvers,
            CurrencyId::Native,
            10_000u32.into(),
            TreasuryOrigin::Kusama,
            Default::default(),
        );
        assert_noop!(
            Grant::edit_grant(
                RuntimeOrigin::signed(*BOB),
                Default::default(),
                None,
                None,
                None,
                None,
                None,
                None
            ),
            Error::<Test>::OnlySubmitterCanEdit
        );
    });
}

#[test]
fn edit_grant_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Grant::edit_grant(
                RuntimeOrigin::signed(*BOB),
                Default::default(),
                None,
                None,
                None,
                None,
                None,
                None
            ),
            Error::<Test>::GrantNotFound
        );
    });
}

#[test]
fn edit_grant_grant_cancelled() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let approvers = get_approvers(10);
        let grant_id: H256 = Default::default();
        let _ = Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            milestones,
            approvers,
            CurrencyId::Native,
            10_000u32.into(),
            TreasuryOrigin::Kusama,
            grant_id,
        );
        let mut grant = PendingGrants::<Test>::get(grant_id).expect("qed");
        grant.is_cancelled = true;

        PendingGrants::<Test>::insert(grant_id, grant);
        assert_noop!(
            Grant::edit_grant(
                RuntimeOrigin::signed(*ALICE),
                Default::default(),
                None,
                None,
                None,
                None,
                None,
                None
            ),
            Error::<Test>::GrantCancelled
        );
    });
}

#[test]
fn edit_with_none_does_not_change_properties() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let approvers = get_approvers(10);
        let grant_id: H256 = Default::default();

        let _ = Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            milestones,
            approvers,
            CurrencyId::Native,
            10_000u32.into(),
            TreasuryOrigin::Kusama,
            grant_id,
        );
        let grant_before = PendingGrants::<Test>::get(grant_id).expect("qed");
        assert_ok!(Grant::edit_grant(
            RuntimeOrigin::signed(*ALICE),
            grant_id,
            None,
            None,
            None,
            None,
            None,
            None
        ));
        let grant_after = PendingGrants::<Test>::get(grant_id).expect("qed");
        assert_eq!(grant_before, grant_after);
    });
}

#[test]
fn assert_properties_are_changed_on_edit() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let approvers = get_approvers(10);
        let grant_id: H256 = Default::default();

        let _ = Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            milestones,
            approvers,
            CurrencyId::Native,
            10_000u32.into(),
            TreasuryOrigin::Kusama,
            grant_id,
        );
        let grant_before = PendingGrants::<Test>::get(grant_id).expect("qed");

        let edited_milestones: BoundedPMilestones<Test> = vec![ProposedMilestoneWithInfo {
            ipfs_hash: [12u8; 32],
            percent: 100,
        }]
        .try_into()
        .expect("qed");

        let edited_approvers: BoundedApprovers<Test> =
            vec![sp_core::sr25519::Public::from_raw([10u8; 32])]
                .try_into()
                .expect("qed");

        let edited_ipfs = [100u8; 32];
        let edited_currency_id = CurrencyId::KSM;
        let edited_amount_requested = 999;
        let edited_treasury_origin = TreasuryOrigin::Imbue;

        assert_ok!(Grant::edit_grant(
            RuntimeOrigin::signed(*ALICE),
            grant_id,
            Some(edited_milestones.clone()),
            Some(edited_approvers.clone()),
            Some(edited_ipfs.clone()),
            Some(edited_currency_id),
            Some(edited_amount_requested),
            Some(edited_treasury_origin)
        ));
        let grant_after = PendingGrants::<Test>::get(&grant_id).expect("qed");

        assert!(grant_before != grant_after);

        // properties that should have changed
        assert_ne!(grant_before.milestones, grant_after.milestones);
        assert_eq!(grant_after.milestones, edited_milestones);

        assert_ne!(grant_before.approvers, grant_after.approvers);
        assert_eq!(grant_after.approvers, edited_approvers);

        assert_ne!(grant_before.ipfs_hash, grant_after.ipfs_hash);
        assert_eq!(grant_after.ipfs_hash, edited_ipfs);

        assert_ne!(grant_before.currency_id, grant_after.currency_id);
        assert_eq!(grant_after.currency_id, edited_currency_id);

        assert_ne!(grant_before.amount_requested, grant_after.amount_requested);
        assert_eq!(grant_after.amount_requested, edited_amount_requested);

        assert_ne!(grant_before.treasury_origin, grant_after.treasury_origin);
        assert_eq!(grant_after.treasury_origin, edited_treasury_origin);

        // properties that should be the same
        assert_eq!(grant_before.created_on, grant_after.created_on);
        assert_eq!(grant_before.is_cancelled, grant_after.is_cancelled);
        assert_eq!(grant_before.is_converted, grant_after.is_converted);
    });
}

#[test]
fn assert_edit_fails_if_milestones_sum_less_than_100() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let approvers = get_approvers(10);
        let grant_id: H256 = Default::default();

        let _ = Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            milestones,
            approvers,
            CurrencyId::Native,
            10_000u32.into(),
            TreasuryOrigin::Kusama,
            grant_id,
        );
        let edited_milestones: BoundedPMilestones<Test> = vec![ProposedMilestoneWithInfo {
            ipfs_hash: [12u8; 32],
            percent: 99,
        }]
        .try_into()
        .expect("qed");

        assert_noop!(
            Grant::edit_grant(
                RuntimeOrigin::signed(*ALICE),
                grant_id,
                Some(edited_milestones),
                None,
                None,
                None,
                None,
                None
            ),
            Error::<Test>::MustSumTo100
        );
    });
}
#[test]
fn success_grant_creation() {
    new_test_ext().execute_with(|| {
        assert_ok!(Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            get_milestones(5),
            get_approvers(5),
            CurrencyId::Native,
            10_000u32.into(),
            TreasuryOrigin::Kusama,
            Default::default()
        ));
    });
}

#[test]
fn success_cancel_grant_as_authority() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        create_native_default_grant(grant_id, *ALICE);
        assert_noop!(
            Grant::cancel_grant(RuntimeOrigin::signed(*BOB), grant_id, true),
            BadOrigin
        );
        assert_ok!(Grant::cancel_grant(RuntimeOrigin::root(), grant_id, true));
    });
}

#[test]
fn success_cancel_grant_as_submitter() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        create_native_default_grant(grant_id, *ALICE);
        assert_ok!(Grant::cancel_grant(
            RuntimeOrigin::signed(*ALICE),
            grant_id,
            false
        ));
    });
}

#[test]
fn cancel_grant_not_submitter() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        create_native_default_grant(grant_id, *ALICE);
        assert_noop!(
            Grant::cancel_grant(RuntimeOrigin::signed(*BOB), grant_id, false),
            Error::<Test>::OnlySubmitterCanEdit
        );
    });
}

#[test]
fn convert_to_proposal_cancelled() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        create_native_default_grant(grant_id, *ALICE);
        let _ = Grant::cancel_grant(RuntimeOrigin::root(), grant_id, true);

        assert_noop!(
            Grant::convert_to_milestones(RuntimeOrigin::signed(*ALICE), grant_id),
            Error::<Test>::GrantCancelled
        );
    });
}

#[test]
fn convert_to_proposal_not_submitter() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        create_native_default_grant(grant_id, *ALICE);
        assert_noop!(
            Grant::convert_to_milestones(RuntimeOrigin::signed(*BOB), grant_id),
            Error::<Test>::OnlySubmitterCanEdit
        );
    });
}

#[test]
fn convert_to_proposal_already_converted() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        create_native_default_grant(grant_id, *ALICE);
        assert_ok!(Grant::convert_to_milestones(
            RuntimeOrigin::signed(*ALICE),
            grant_id
        ));
        assert_noop!(
            Grant::convert_to_milestones(RuntimeOrigin::signed(*ALICE), grant_id),
            Error::<Test>::AlreadyConverted
        );
    });
}

#[test]
fn e2e() {
    new_test_ext().execute_with(|| {
        assert!(false);
    });
}

fn get_milestones(mut n: u32) -> BoundedPMilestones<Test> {
    let max = <Test as Config>::MaxMilestonesPerGrant::get();
    if n > max {
        n = max;
    }
    let percent = 100 / n;
    (0..n)
        .map(|i| ProposedMilestoneWithInfo {
            percent: percent.try_into().expect("qed"),
            ipfs_hash: [i as u8; 32],
        })
        .collect::<Vec<ProposedMilestoneWithInfo>>()
        .try_into()
        .expect("qed")
}

fn get_approvers(mut n: u32) -> BoundedApprovers<Test> {
    let max = <Test as Config>::MaxApprovers::get();
    if n > max {
        n = max;
    }
    (0..n)
        .map(|i| sp_core::sr25519::Public::from_raw([i as u8; 32]))
        .collect::<Vec<sp_core::sr25519::Public>>()
        .try_into()
        .expect("qed")
}

fn create_native_default_grant(grant_id: GrantId, submitter: AccountId) {
    assert_ok!(Grant::submit_initial_grant(
        RuntimeOrigin::signed(submitter),
        Default::default(),
        get_milestones(10),
        get_approvers(10),
        CurrencyId::Native,
        10_000u32.into(),
        TreasuryOrigin::Kusama,
        grant_id,
    ));
}
