use crate::mock::*;
use crate::*;
use common_types::CurrencyId;
use frame_support::pallet_prelude::Hooks;
use frame_support::{assert_noop, assert_ok, once_cell::sync::Lazy};
use sp_core::H256;
use sp_runtime::DispatchError::BadOrigin;
use sp_std::collections::btree_map::BTreeMap;

pub fn gen_hash(seed: u8) -> BriefHash {
    H256::from([seed; 32])
}

#[test]
fn approve_freelancer_not_root() {
    build_test_externality().execute_with(|| {
        assert_noop!(
            BriefsMod::add_to_fellowship(RuntimeOrigin::signed(*ALICE), *BOB),
            BadOrigin
        );
    });
}

#[test]
fn approve_freelancer_as_root() {
    build_test_externality().execute_with(|| {
        assert_ok!(BriefsMod::add_to_fellowship(RuntimeOrigin::root(), *BOB));
    });
}

#[test]
fn create_brief_not_approved_applicant() {
    build_test_externality().execute_with(|| {
        assert_noop!(
            BriefsMod::create_brief(
                RuntimeOrigin::signed(*BOB),
                get_brief_owners(1),
                *ALICE,
                100000,
                10000,
                gen_hash(1),
                CurrencyId::Native,
                get_milestones(10),
            ),
            Error::<Test>::OnlyApprovedAccountPermitted
        );
    });
}

#[test]
fn create_brief_brief_owner_overflow() {
    build_test_externality().execute_with(|| {
        let _ = BriefsMod::add_to_fellowship(RuntimeOrigin::root(), *ALICE);

        assert_noop!(
            BriefsMod::create_brief(
                RuntimeOrigin::signed(*BOB),
                get_brief_owners(u32::MAX),
                *ALICE,
                100000,
                10000,
                gen_hash(1),
                CurrencyId::Native,
                get_milestones(10),
            ),
            Error::<Test>::TooManyBriefOwners
        );
    });
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        BriefsMod::on_initialize(System::block_number());
    }
}

fn get_brief_owners(mut n: u32) -> BoundedBriefOwners<Test> {
    let max = <Test as Config>::MaxBriefOwners::get();
    if n > max {
        n = max;
    }
    (0..n)
        .map(|i| AccountId::from_raw([n as u8; 32]))
        .collect::<Vec<AccountId>>()
        .try_into()
        .expect("qed")
}

fn get_milestones(mut n: u32) -> BoundedBriefMilestones<Test> {
    let max = <Test as Config>::MaxMilestones::get();
    if n > max {
        n = max
    }
    let mut btree_map: BoundedBriefMilestones<Test> = BTreeMap::new().try_into().expect("qed");

    (0..n)
        .map(|i|{
            btree_map.try_insert(
                i,
                BriefMilestone {
                    milestone_key: i,
                    percentage_to_unlock: 100/i,
                    name: vec![i as u8].try_into().expect("qed")
                }
            ).expect("qed")
        }).collect::<Vec<_>>();
    btree_map
}
