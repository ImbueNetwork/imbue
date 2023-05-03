use crate::tests::{get_approvers, get_milestones};
use crate::{mock::*, tests};

use common_types::{CurrencyId, TreasuryOrigin};
use frame_support::assert_ok;
use orml_traits::MultiCurrency;
use pallet_proposals::{Projects, ProposedMilestone, RoundType};
use sp_core::bounded_vec;

#[test]
fn create_proposal_from_grant() {
    new_test_ext().execute_with(|| {
        let grant_id = gen_grant_id(100);
        let contribution_value: Balance = 10_000;

        assert_ok!(Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            get_milestones(10),
            get_approvers(10),
            CurrencyId::Native,
            contribution_value,
            TreasuryOrigin::Imbue,
            grant_id,
        ));
        assert_ok!(Grant::convert_to_project(
            RuntimeOrigin::signed(*ALICE),
            grant_id
        ));
        let created_project = Projects::<Test>::get(1).unwrap();
        assert_eq!(created_project.agreement_hash, grant_id);
        assert_eq!(created_project.approved_for_funding, true);
        assert_eq!(created_project.required_funds, contribution_value);
    });
}

#[test]
fn assert_state_from_grant_conversion_is_same_as_proposal() {
    new_test_ext().execute_with(|| {
        let grant_id = gen_grant_id(100);
        let milestones = get_milestones(10);
        let approvers = bounded_vec![*BOB];
        let contribution_value: Balance = 10_000;
        let project_grant = 1;
        let project_normal = 2;

        // create a proposal from pallet-grants
        assert_ok!(Grant::submit_initial_grant(
            RuntimeOrigin::signed(*ALICE),
            Default::default(),
            milestones.clone(),
            approvers,
            CurrencyId::Native,
            contribution_value,
            TreasuryOrigin::Imbue,
            grant_id,
        ));
        assert_ok!(Grant::convert_to_project(
            RuntimeOrigin::signed(*ALICE),
            grant_id
        ));

        // create a proposal with same parameters
        assert_ok!(Proposals::create_project(
            RuntimeOrigin::signed(*ALICE),
            grant_id,
            milestones
                .into_iter()
                .map(|m| ProposedMilestone {
                    percentage_to_unlock: m.percent as u32
                })
                .collect::<Vec<ProposedMilestone>>()
                .try_into()
                .unwrap(),
            contribution_value,
            CurrencyId::Native,
        ));
        assert_ok!(Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 5,
            bounded_vec![project_normal],
            RoundType::ContributionRound,
        ));

        tests::run_to_block(System::block_number() + 1);

        assert_ok!(Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_normal,
            contribution_value,
        ));

        assert_ok!(Proposals::approve(
            RuntimeOrigin::root(),
            Some(1),
            project_normal,
            None
        ));

        let p_grant = Projects::<Test>::get(project_grant).unwrap();
        let p_normal = Projects::<Test>::get(project_normal).unwrap();

        let p_grant_id = Proposals::project_account_id(project_grant);
        let balance_grant = Tokens::free_balance(CurrencyId::Native, &p_grant_id);

        let p_normal_id = Proposals::project_account_id(project_grant);
        let balance_normal = Tokens::free_balance(CurrencyId::Native, &p_normal_id);

        assert_eq!(balance_grant, balance_normal);

        assert_eq!(p_grant.agreement_hash, grant_id);
        assert_eq!(p_grant.agreement_hash, p_normal.agreement_hash);

        assert_eq!(
            p_grant.milestones.values().len(),
            p_normal.milestones.values().len()
        );
        let contributions_normal = p_normal.contributions.values().collect::<Vec<_>>();
        assert!(p_grant
            .contributions
            .values()
            .all(|c| contributions_normal.contains(&c)));

        assert_eq!(p_grant.currency_id, p_normal.currency_id);
        assert_eq!(p_grant.required_funds, p_normal.required_funds);
        assert_eq!(p_grant.withdrawn_funds, p_normal.withdrawn_funds);
        assert_eq!(p_grant.raised_funds, p_normal.raised_funds);
        assert_eq!(p_grant.initiator, p_normal.initiator);
        assert_eq!(p_grant.created_on, p_normal.created_on);

        assert_eq!(p_grant.approved_for_funding, true);
        assert_eq!(p_grant.approved_for_funding, p_normal.approved_for_funding);

        assert_eq!(p_grant.funding_threshold_met, true);
        assert_eq!(
            p_grant.funding_threshold_met,
            p_normal.funding_threshold_met
        );

        assert_eq!(p_grant.cancelled, false);
        assert_eq!(p_grant.cancelled, p_normal.cancelled);
    });
}
