use crate::mock::*;
use crate::*;
use common_types::CurrencyId;
use frame_support::pallet_prelude::Hooks;
use frame_support::{assert_noop, assert_ok, once_cell::sync::Lazy, bounded_vec};
use sp_core::H256;
use sp_runtime::DispatchError::BadOrigin;
use sp_std::collections::btree_map::BTreeMap;
use std::convert::TryInto;
use pallet_proposals::RoundType;
use crate::tests::{get_milestones};

// all the integration tests for a brief to proposal conversion
#[test]
fn create_proposal_from_brief() {
    build_test_externality().execute_with(|| {        
        let brief_id = H256::from([12; 32]);
        let contribution_value: Balance = 10000;
        
        let _ = BriefsMod::create_brief(
            RuntimeOrigin::signed(*BOB),
            tests::get_brief_owners(1),
            *ALICE,
            contribution_value,
            contribution_value,
            brief_id.clone(),
            CurrencyId::Native,
            tests::get_milestones(10),
        );

        assert_ok!(BriefsMod::commence_work(
            RuntimeOrigin::signed(*ALICE),
            brief_id
        ));
    });
}

#[test]
fn assert_state_from_brief_conversion_is_same_as_proposals_flow() {
    build_test_externality().execute_with(|| {        
        let brief_id = H256::from([12; 32]);
        let milestones = get_milestones(10);
        let mut project_key = 1;
        let contribution_value: Balance = 10000;
        // This is the minimum path to a proposal from the briefs pallet.
        let _ = BriefsMod::create_brief(
            RuntimeOrigin::signed(*ALICE),
            tests::get_brief_owners(1),
            *BOB,
            contribution_value,
            contribution_value,
            brief_id.clone(),
            CurrencyId::Native,
            milestones.clone(),
        );
        
        let _ = BriefsMod::commence_work(
            RuntimeOrigin::signed(*ALICE),
            brief_id
        );

        // Now we create a proposal with the same parameters.
        // The state of each of the project should be the same and therefore will function the same.
        Proposals::create_project(
            RuntimeOrigin::signed(*ALICE),
            brief_id,
            milestones
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .try_into()
                .expect("proposed milestones are too long"),
            contribution_value,
            CurrencyId::Native,
        );

        let _ = Proposals::schedule_round(
            RuntimeOrigin::root(),
            System::block_number(),
            System::block_number() + 100,
            bounded_vec![0u32],
            RoundType::ContributionRound,
        )
        .unwrap();
        
        let _ = Proposals::contribute(
            RuntimeOrigin::signed(*BOB),
            Some(1),
            project_key + 1,
            contribution_value,
        );
        let _ = Proposals::approve(RuntimeOrigin::root(), Some(1), project_key, Some(milestones.keys().cloned().collect::<Vec<_>>().try_into().expect("qed"))).unwrap();

    });
}