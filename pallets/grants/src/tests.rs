#[allow(unused)]
use crate::mock::*;
use crate::pallet::{BoundedApprovers, BoundedPMilestones, Config, Error};
use common_types::{CurrencyId, TreasuryOrigin};
use frame_support::{assert_noop, pallet_prelude::*};
use pallet_proposals::ProposedMilestone;
use sp_arithmetic::per_things::Percent;
use sp_core::H256;

#[test]
fn ensure_milestone_percent_equal_100() {
    new_test_ext().execute_with(|| {
        let milestones: BoundedPMilestones<Test> = vec![ProposedMilestone {
            percentage_to_unlock: Percent::from_percent(50u8),
        }]
        .try_into()
        .expect("qed");

        assert_noop!(
            Grant::create_and_convert(
                RuntimeOrigin::signed(*ALICE),
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

        let _ = Grant::create_and_convert(
            RuntimeOrigin::signed(*ALICE),
            milestones,
            approvers,
            CurrencyId::Native,
            10_000u32.into(),
            TreasuryOrigin::Kusama,
            grant_id,
        );
        assert_noop!(
            Grant::create_and_convert(
                RuntimeOrigin::signed(*ALICE),
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

pub(crate) fn get_milestones(mut n: u32) -> BoundedPMilestones<Test> {
    let max = <Test as Config>::MaxMilestonesPerGrant::get();
    if n > max {
        n = max;
    }
    let percent = Percent::from_percent((100 / n) as u8);
    (0..n)
        .map(|_m| ProposedMilestone {
            percentage_to_unlock: percent,
        })
        .collect::<Vec<ProposedMilestone>>()
        .try_into()
        .expect("qed")
}

pub(crate) fn get_approvers(mut n: u32) -> BoundedApprovers<Test> {
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

pub(crate) fn _run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Proposals::on_initialize(System::block_number());
    }
}
