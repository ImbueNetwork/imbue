use crate::mock::*;
use frame_support::{assert_ok, assert_noop, pallet_prelude::*};
use crate::pallet::{ProposedMilestoneWithInfo, BoundedPMilestones, Config, BoundedApprovers, Error, PendingGrants};
use common_types::{TreasuryOrigin, CurrencyId};
use sp_core::H256;
use sp_runtime::DispatchError::BadOrigin;

#[test]
fn ensure_milestone_percent_equal_100() {
    new_test_ext().execute_with(|| {
        
        let milestones: BoundedPMilestones<Test> = vec![ ProposedMilestoneWithInfo  {
            percent: 50u8,
            ipfs_hash: Default::default(),
        }].try_into().expect("qed");
        
        assert_noop!(Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), Default::default(), milestones, get_approvers(5), CurrencyId::Native, 10_000u32.into(), TreasuryOrigin::Kusama, Default::default()), Error::<Test>::MustSumTo100);
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

        let _ = Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), Default::default(), milestones, approvers, CurrencyId::Native, 10_000u32.into(), TreasuryOrigin::Kusama, grant_id);
        assert_noop!(Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), Default::default(), milestones2, approvers2, CurrencyId::Native, 10_000u32.into(), TreasuryOrigin::Kusama, grant_id), Error::<Test>::GrantAlreadyExists);
    });
}

#[test]
fn edit_grant_only_submitter_can_edit() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let approvers = get_approvers(10);
        let _ = Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), Default::default(), milestones, approvers, CurrencyId::Native, 10_000u32.into(), TreasuryOrigin::Kusama, Default::default());
        assert_noop!(Grant::edit_grant(RuntimeOrigin::signed(*BOB), Default::default(), None, None, None, None, None), Error::<Test>::OnlySubmitterCanEdit);
    });
}

#[test]
fn edit_grant_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(Grant::edit_grant(RuntimeOrigin::signed(*BOB), Default::default(), None, None, None, None, None), Error::<Test>::GrantNotFound);
    });
}

#[test]
fn edit_grant_grant_cancelled() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let approvers = get_approvers(10);  
        let grant_id: H256 = Default::default();
        let _ = Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), Default::default(), milestones, approvers, CurrencyId::Native, 10_000u32.into(), TreasuryOrigin::Kusama, grant_id);
        let mut grant = PendingGrants::<Test>::get(grant_id).expect("qed");
        grant.is_cancelled = true;

        PendingGrants::<Test>::insert(grant_id, grant);
        assert_noop!(Grant::edit_grant(RuntimeOrigin::signed(*ALICE), Default::default(), None, None, None, None, None), Error::<Test>::GrantCancelled);

    });
}

#[test]
fn edit_with_none_does_not_change_properties() {
    new_test_ext().execute_with(|| {
        let milestones = get_milestones(10);
        let approvers = get_approvers(10);
        let grant_id: H256 = Default::default();

        let _ = Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), Default::default(), milestones, approvers, CurrencyId::Native, 10_000u32.into(), TreasuryOrigin::Kusama, grant_id);
        let grant_before = PendingGrants::<Test>::get(grant_id).expect("qed");
        assert_ok!(Grant::edit_grant(RuntimeOrigin::signed(*ALICE), grant_id, None, None, None, None, None,));
        let grant_after = PendingGrants::<Test>::get(grant_id).expect("qed");
        assert_eq!(grant_before, grant_after);
    });
}

#[test]
fn assert_properties_are_changed_on_edit() {
    new_test_ext().execute_with(|| {
        assert!(false);
    });
}


#[test]
fn success_grant_creation() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), 
                Default::default(), 
                get_milestones(5), get_approvers(5), 
                CurrencyId::Native, 
                10_000u32.into(), 
                TreasuryOrigin::Kusama, 
                Default::default()
            ) 
        );
    });
}

#[test]
fn success_cancel_grant_as_authority() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        let _ = Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), 
                Default::default(), 
                get_milestones(5), get_approvers(5), 
                CurrencyId::Native, 
                10_000u32.into(), 
                TreasuryOrigin::Kusama, 
                grant_id,
            );
        assert_noop!(Grant::cancel_grant(RuntimeOrigin::signed(*BOB), grant_id, true), BadOrigin);
        assert_ok!(Grant::cancel_grant(RuntimeOrigin::root(), grant_id, true));
    });
}

#[test]
fn success_cancel_grant_as_submitter() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        let _ = Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), 
                Default::default(), 
                get_milestones(5), get_approvers(5), 
                CurrencyId::Native, 
                10_000u32.into(), 
                TreasuryOrigin::Kusama, 
                grant_id,
            );
        assert_ok!(Grant::cancel_grant(RuntimeOrigin::signed(*ALICE), grant_id, false));
    });
}

#[test]
fn cancel_grant_not_submitter() {
    new_test_ext().execute_with(|| {
        let grant_id = Default::default();
        let _ = Grant::submit_initial_grant(RuntimeOrigin::signed(*ALICE), 
                Default::default(), 
                get_milestones(5), get_approvers(5), 
                CurrencyId::Native, 
                10_000u32.into(), 
                TreasuryOrigin::Kusama, 
                grant_id,
            );
        assert_noop!(Grant::cancel_grant(RuntimeOrigin::signed(*BOB), grant_id, false), Error::<Test>::OnlySubmitterCanEdit);
    });
}

#[test]
fn convert_to_proposal_cancelled() {
    new_test_ext().execute_with(|| {
        assert!(false);
    });
}

#[test]
fn convert_to_proposal_not_submitter() {
    new_test_ext().execute_with(|| {
        assert!(false);
    });
}

#[test]
fn convert_to_proposal_already_converted() {
    new_test_ext().execute_with(|| {
        assert!(false);
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
    (0..n).map(|i| {
        ProposedMilestoneWithInfo {
            percent: percent.try_into().expect("qed"), 
            ipfs_hash: [i as u8; 32],
        }
    }).collect::<Vec<ProposedMilestoneWithInfo>>().try_into().expect("qed")  
}

fn get_approvers(mut n: u32) -> BoundedApprovers<Test> {
    let max = <Test as Config>::MaxApprovers::get();
    if n > max {
        n = max;
    }
    (0..n).map(|i| {
        sp_core::sr25519::Public::from_raw([i as u8; 32])
    }).collect::<Vec<sp_core::sr25519::Public>>().try_into().expect("qed")
}
