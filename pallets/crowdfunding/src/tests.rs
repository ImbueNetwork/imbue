use crate::{mock::*, Error, Event, Config, CrowdFundKey, CrowdFundCount};
use frame_support::{assert_noop, assert_ok};
use pallet_proposals::{
	ProposedMilestone,
};
use crate::pallet::BoundedProposedMilestones;
use sp_core::H256;
use common_types::CurrencyId;
use test_utils::*;
use sp_runtime::DispatchError::BadOrigin;

pub(crate) mod test_utils {
	use super::*;

	pub fn get_milestones(mut n: u32) -> BoundedProposedMilestones<Test> {
		let max = <Test as Config>::MaxMilestonesPerCrowdFund::get();
		if n > max {
			n = max
		}
		(0..n).map(|_| {
			ProposedMilestone {
				percentage_to_unlock: 100u32 / n
			}
		}).collect::<Vec<ProposedMilestone>>().try_into().expect("bound is ensured; qed")
	}

	pub fn get_hash(n: u8) -> H256 {
		H256::from([n; 32])
	}

	pub fn create_cf_default(who: AccountId, amount: Balance) -> CrowdFundKey {
		let key = CrowdFundCount::<Test>::get();
		assert_ok!(CrowdFunding::create_crowdfund(RuntimeOrigin::signed(who), get_hash(2u8), get_milestones(10), amount, CurrencyId::Native));
		key
	}

	pub fn create_cf_default_and_contribute(creator: AccountId, contributors: Vec<AccountId>, amount: Balance) {
		let cf_key = create_cf_default(creator, amount);
		assert_ok!(CrowdFunding::open_contributions(RuntimeOrigin::root(), cf_key));
		let divided = amount / contributors.len() as u64;
		contributors.iter().for_each(|acc| {
			assert_ok!(CrowdFunding::contribute(RuntimeOrigin::signed(*acc), cf_key, divided));			
		})
	}

	pub fn run_to_block(n: BlockNumber) {
		while System::block_number() < n {
			System::set_block_number(System::block_number() + 1);
			System::on_initialize(System::block_number());
			CrowdFunding::on_initialize(System::block_number());
		}
	}
}

#[test]
fn crowdfund_key_actually_increments_lol() {
	new_test_ext().execute_with(|| {
		let one = CrowdFundCount::<Test>::get();
		create_cf_default(*ALICE, 100_000u64);
		let two = CrowdFundCount::<Test>::get();
		assert_eq!(two - 1, one);
		let three = CrowdFundCount::<Test>::get();
		create_cf_default(*BOB, 100_000u64);
		assert_eq!(three - 1, two);
	});
}


#[test]
fn milestones_must_sum_to_100_on_creation() {
	new_test_ext().execute_with(|| {
		let milestones_under_100: BoundedProposedMilestones<Test> = vec![ProposedMilestone{percentage_to_unlock: 50}].try_into().expect("qed");
		assert_noop!(
			CrowdFunding::create_crowdfund(RuntimeOrigin::signed(*ALICE), get_hash(1u8), milestones_under_100, 100_000, CurrencyId::Native),
		 Error::<Test>::MilestonesTotalPercentageMustEqual100);
	});
}

#[test]
fn below_minimum_required_funds() {
	new_test_ext().execute_with(|| {
		let below_minimum_required: Balance = <Test as Config>::MinimumRequiredFunds::get() - 10;
		assert_noop!(CrowdFunding::create_crowdfund(RuntimeOrigin::signed(*ALICE), get_hash(1u8), get_milestones(10), 100_000, CurrencyId::Native), Error::<Test>::BelowMinimumRequiredFunds);
	});
}

#[test]
fn update_crowdfund_ms_must_sum_to_100() {
	new_test_ext().execute_with(|| {
		create_cf_default(*ALICE, 100_000u64);
		let milestones_under_100: BoundedProposedMilestones<Test> = vec![ProposedMilestone{percentage_to_unlock: 50}].try_into().expect("qed");
		assert_noop!(CrowdFunding::update_crowdfund(
			RuntimeOrigin::signed(*ALICE),
			0u32,
			Some(milestones_under_100),
			None,
			None,
			None,
		), Error::<Test>::MilestonesTotalPercentageMustEqual100);
	});
}

#[test]
fn update_crowdfund_already_approved() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn update_crowdfund_already_converted() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn update_crowdfund_already_cancelled() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn update_crowdfund_not_initiator() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn update_crowdfund_none_values_dont_mutate() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn update_crowdfund_actually_mutates() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn open_contributions_already_in_round() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000u64);
		assert_ok!(CrowdFunding::open_contributions(RuntimeOrigin::root(), key));
		assert_noop!(CrowdFunding::open_contributions(RuntimeOrigin::root(), key), Error::<Test>::AlreadyInContributionRound);
	});
}

#[test]
fn open_contributions_not_authority() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000u64);
		assert_noop!(CrowdFunding::open_contributions(RuntimeOrigin::signed(*ALICE), key), Error::<Test>::BadOrigin);
	});
}

#[test]
fn open_contributions_approves_for_funding() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000u64);
		assert_ok!(CrowdFunding::open_contributions(RuntimeOrigin::root(), key));
		let cf = CrowdFunds::<Test>::get(key).expect("oops");
		ensure!(cf.approved_for_funding);
	});
}


#[test]
fn open_contributions_crowdfund_doesnt_exist_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(CrowdFunding::open_contributions(RuntimeOrigin::root(), key), Error::<Test>::CrowdFundDoesNotExist);
	});
}

#[test]
fn cannot_contribute_before_round_start() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000u64);
		assert_noop!(CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, 100_000), Error::<Test>::ContributionRoundNotStarted);
	});
}

#[test]
fn cannot_contribute_after_round_end() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000u64);
		assert_ok!(CrowdFunding::open_contributions(RuntimeOrigin::root(), key));
		let expiry_block = frame_system::Pallet::<Test>::block_number() + <Test as Config>::RoundExpiry::get();
		run_to_block(expiry_block + 1);
		assert_noop!(CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, 100_000), Error::<Test>::ContributionRoundNotStarted);
	});
}

#[test]
fn new_contribute_zero_contribution() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn new_contribute_doesnt_exist() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn multiple_contributions_sum_contribution() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn contribution_reserves_balance() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn contribution_mutates_raised_funds_and_pushes_contribution() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn ensure_initiator_works() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn do_approve_not_authority() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn do_approve_raised_funds_not_reached() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn do_approve_does_not_exist() {
	new_test_ext().execute_with(|| {
		assert!(false)

	});
}

#[test]
fn on_initialize_removes_storage_flags() {
	new_test_ext().execute_with(|| {
		assert!(false)
	});
}

