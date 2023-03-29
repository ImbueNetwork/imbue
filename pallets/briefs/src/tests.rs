use crate as briefs;
use crate::mock::*;
use crate::*;
use common_types::CurrencyId;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use proposals::ProposedMilestone;
use sp_core::H256;
use sp_runtime::DispatchError::BadOrigin;
use sp_std::collections::btree_map::BTreeMap;
use std::convert::TryInto;

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
        // todo!()
        // Only accounts in the fellowship can apply for work
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

#[test]
fn create_brief_with_no_contribution_ok() {
    build_test_externality().execute_with(|| {
        assert_ok!(BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            get_brief_owners(1),
            *ALICE,
            100000,
            10,
            gen_hash(1),
            CurrencyId::Native,
            get_milestones(10),
        ));
    });
}

#[test]
fn create_brief_no_contribution_and_contribute() {
    build_test_externality().execute_with(|| {
        let brief_id = gen_hash(1);
        let contribution_value = 1000;
        let bob_initial_balance = Tokens::free_balance(CurrencyId::Native, &*BOB);

        assert_ok!(BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            get_brief_owners(1),
            *ALICE,
            100000,
            0,
            brief_id,
            CurrencyId::Native,
            get_milestones(10),
        ));

        (0..5).for_each(|_| {
            assert_ok!(BriefsMod::contribute_to_brief(
                RuntimeOrigin::signed(*BOB),
                brief_id,
                contribution_value,
            ));
        });

        let latest_event = <frame_system::Pallet<Test>>::events()
            .pop()
            .expect("Expected at least one RuntimeEventRecord to be found")
            .event;

        assert_eq!(
            latest_event,
            mock::RuntimeEvent::from(briefs::Event::BriefContribution(brief_id))
        );

        assert_eq!(
            Tokens::free_balance(CurrencyId::Native, &*BOB),
            bob_initial_balance - contribution_value.saturating_mul(5)
        );
    });
}

#[test]
fn contribute_to_brief_not_brief_owner() {
    build_test_externality().execute_with(|| {
        let brief_id = gen_hash(1);
        let contribution_value = 1000;
        assert_ok!(BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            get_brief_owners(1),
            *ALICE,
            100000,
            100,
            brief_id,
            CurrencyId::Native,
            get_milestones(10),
        ));

        assert_noop!(
            BriefsMod::contribute_to_brief(
                RuntimeOrigin::signed(*ALICE),
                brief_id,
                contribution_value,
            ),
            Error::<Test>::NotAuthorised
        );
    });
}

#[test]
fn contribute_to_brief_more_than_total_ok() {
    build_test_externality().execute_with(|| {
        let brief_id = gen_hash(1);
        let contribution_value = 1000;

        assert_ok!(BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            get_brief_owners(1),
            *ALICE,
            contribution_value,
            contribution_value,
            brief_id,
            CurrencyId::Native,
            get_milestones(10),
        ));
        assert_ok!(BriefsMod::contribute_to_brief(
            RuntimeOrigin::signed(*BOB),
            brief_id,
            contribution_value,
        ));
    });
}

#[test]
fn create_brief_already_exists() {
    build_test_externality().execute_with(|| {
        let brief_id = gen_hash(1);
        let contribution_value = 1000;

        assert_ok!(BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            get_brief_owners(1),
            *ALICE,
            contribution_value,
            contribution_value,
            brief_id,
            CurrencyId::Native,
            get_milestones(10),
        ));

        assert_noop!(
            BriefsMod::create_brief(
                RuntimeOrigin::signed(*BOB),
                get_brief_owners(1),
                *ALICE,
                contribution_value,
                contribution_value,
                brief_id,
                CurrencyId::Native,
                get_milestones(10),
            ),
            Error::<Test>::BriefAlreadyExists
        );
    });
}

#[test]
fn only_applicant_can_start_work() {
    build_test_externality().execute_with(|| {
        let brief_id = gen_hash(1);
        let contribution_value = 1000;

        assert_ok!(BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            get_brief_owners(1),
            *ALICE,
            contribution_value,
            contribution_value,
            brief_id,
            CurrencyId::Native,
            get_milestones(10),
        ));

        assert_noop!(
            BriefsMod::commence_work(RuntimeOrigin::signed(*BOB), brief_id,),
            Error::<Test>::NotAuthorised
        );

        assert_ok!(BriefsMod::commence_work(
            RuntimeOrigin::signed(*ALICE),
            brief_id,
        ));
    });
}

#[test]
fn initial_contribution_and_extra_contribution_aggregates() {
    build_test_externality().execute_with(|| {
        assert!(false);
    });
}

#[test]
fn test_remaining_bounty_api() {
    build_test_externality().execute_with(|| {
        assert!(false);
    });
}

pub(crate) fn get_brief_owners(mut n: u32) -> BoundedBriefOwners<Test> {
    let max = <Test as Config>::MaxBriefOwners::get();
    if n > max {
        n = max;
    }
    (0..n)
        .map(|_| AccountId::from_raw([n as u8; 32]))
        .collect::<Vec<AccountId>>()
        .try_into()
        .expect("qed")
}

pub(crate) fn get_milestones(mut n: u32) -> BoundedBriefMilestones<Test> {
    let max = <Test as Config>::MaxMilestones::get();
    if n > max {
        n = max
    }
    let mut btree_map: BoundedBriefMilestones<Test> = BTreeMap::new().try_into().expect("qed");

    let _ = (0..n)
        .map(|i| {
            btree_map
                .try_insert(
                    i,
                    ProposedMilestone {
                        percentage_to_unlock: 100 / n,
                    },
                )
                .expect("qed")
        })
        .collect::<Vec<_>>();
    btree_map
}
