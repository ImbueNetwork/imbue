use crate as briefs;
use crate::mock::*;
use crate::test_utils::gen_hash;
use crate::*;

use common_types::CurrencyId;
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};
use orml_traits::MultiCurrency;
use pallet_proposals::{BoundedProposedMilestones, Projects, ProposedMilestone};
use sp_arithmetic::per_things::Percent;
use sp_runtime::DispatchError::BadOrigin;
use std::convert::TryInto;

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
        // TODO:
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
            mock::RuntimeEvent::from(briefs::Event::BriefContribution(*BOB, brief_id))
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
            Error::<Test>::MustBeBriefOwner
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
            Error::<Test>::MustBeApplicant
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

        assert_ok!(BriefsMod::commence_work(
            RuntimeOrigin::signed(*ALICE),
            brief_id,
        ));

        let created_project = Projects::<Test>::get(1).unwrap();
        assert_eq!(
            created_project.raised_funds,
            contribution_value.saturating_mul(2)
        );
    });
}

#[test]
fn reserved_funds_are_transferred_to_project_kitty() {
    build_test_externality().execute_with(|| {
        let brief_id = gen_hash(100);
        let contribution_value: Balance = 10000;

        let _ = BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            tests::get_brief_owners(1),
            *ALICE,
            contribution_value,
            contribution_value,
            brief_id.clone(),
            CurrencyId::Native,
            get_milestones(10),
        );

        assert_ok!(BriefsMod::commence_work(
            RuntimeOrigin::signed(*ALICE),
            brief_id
        ));

        let project_id: AccountId = Proposals::project_account_id(1);
        let project_balance = Tokens::free_balance(CurrencyId::Native, &project_id);
        assert_eq!(project_balance, contribution_value);
    });
}

pub(crate) fn _run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Proposals::on_initialize(System::block_number());
        //BriefsMod::on_initialize(System::block_number());
    }
}

pub(crate) fn get_brief_owners(mut n: u32) -> BoundedBriefOwners<Test> {
    let max = <Test as briefs::Config>::MaxBriefOwners::get();
    if n > max {
        n = max;
    }
    (0..n)
        .map(|_| AccountId::from_raw([n as u8; 32]))
        .collect::<Vec<AccountId>>()
        .try_into()
        .expect("qed")
}

pub(crate) fn get_milestones(mut n: u32) -> BoundedProposedMilestones<Test> {
    let max = <Test as briefs::Config>::MaxMilestonesPerBrief::get();
    if n > max {
        n = max;
    }
    let milestones = (0..n)
        .map(|_| ProposedMilestone {
            percentage_to_unlock: Percent::from_percent((100 / n) as u8),
        })
        .collect::<Vec<ProposedMilestone>>()
        .try_into()
        .expect("qed");

    milestones
}
