use crate::mock::*;
use crate::tests::get_milestones;
use crate::*;
use common_types::CurrencyId;

use frame_support::{assert_ok, bounded_vec};
use pallet_proposals::{Projects, RoundType};
use sp_core::H256;
use std::convert::TryInto;


// all the integration tests for a brief to proposal conversion
#[test]
fn create_proposal_from_brief() {
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

        let created_project = Projects::<Test>::get(1).unwrap();

        assert_eq!(created_project.agreement_hash, brief_id);
        assert_eq!(created_project.approved_for_funding, true);
        assert_eq!(created_project.required_funds, contribution_value);
    });
}

#[test]
fn assert_state_from_brief_conversion_is_same_as_proposals_flow() {
    build_test_externality().execute_with(|| {        
        let brief_id = H256::from([12; 32]);
        let milestones = get_milestones(10);
        let project_key = 1;
        let contribution_value: Balance = 10000;
        // This is the minimum path to a proposal from the briefs pallet.
        let _ = BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            tests::get_brief_owners(1),
            *ALICE,
            contribution_value,
            contribution_value,
            brief_id.clone(),
            CurrencyId::Native,
            milestones.clone(),
        ).unwrap();
        
        let _ = BriefsMod::commence_work(
            RuntimeOrigin::signed(*ALICE),
            brief_id
        ).unwrap();

        // Now we create a proposal with the same parameters.
        // The state of each of the project should be the same and therefore will function the same.
        let _ = Proposals::create_project(
            RuntimeOrigin::signed(*ALICE),
            brief_id,
            milestones.clone(),
            contribution_value,
            CurrencyId::Native,
        ).unwrap();


        let _ = Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 5,
            bounded_vec![project_key + 1],
            RoundType::ContributionRound,
        )
        .unwrap();

        tests::run_to_block(System::block_number() + 1u64);

        let _ = Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key + 1,
            contribution_value,
        );

        let _ = Proposals::approve(RuntimeOrigin::root(), Some(1), project_key + 1, Some((0u32..milestones.len() as u32).collect::<Vec<u32>>().try_into().expect("qed"))).unwrap();

        dbg!(Projects::<Test>::iter_keys().collect::<Vec<_>>());
        let brief_p = Projects::<Test>::get(project_key).unwrap();
        let standard_p = Projects::<Test>::get(project_key + 1).unwrap();


        //assert!(brief_p.milestones.values().all(|v| standard_p.milestones.values().contains(v)));
        //assert!(brief_p.contributions.values().iter().all(|v| standard_p.contributions.values().iter().contains(v)));
        assert_eq!(brief_p.currency_id, standard_p.currency_id);
        assert_eq!(brief_p.required_funds, standard_p.required_funds);
        assert_eq!(brief_p.withdrawn_funds, standard_p.withdrawn_funds);
        assert_eq!(brief_p.raised_funds, standard_p.raised_funds);
        assert_eq!(brief_p.initiator, standard_p.initiator);
        assert_eq!(brief_p.created_on, standard_p.created_on);
        assert_eq!(brief_p.approved_for_funding, true);
        assert_eq!(brief_p.approved_for_funding, standard_p.approved_for_funding);
        assert_eq!(brief_p.funding_threshold_met, true);
        assert_eq!(brief_p.funding_threshold_met, standard_p.funding_threshold_met);
        assert_eq!(brief_p.cancelled, false);
        assert_eq!(brief_p.cancelled, standard_p.cancelled);
    });
}