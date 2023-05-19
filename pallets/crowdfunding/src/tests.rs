use crate::{mock::*, Error, Event, Config, CrowdFundKey, CrowdFundCount, CrowdFunds, CrowdFundsInRound, RoundsExpiring, RoundType};
use frame_support::{assert_noop, assert_ok, traits::Hooks};
use pallet_proposals::{
	ProposedMilestone,
};
use crate::pallet::BoundedProposedMilestones;
use sp_core::H256;
use common_types::CurrencyId;
use test_utils::*;
use sp_runtime::DispatchError::BadOrigin;
use orml_traits::MultiReservableCurrency;

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

	pub fn create_cf_default_and_contribute(creator: AccountId, contributors: Vec<AccountId>, amount: Balance) -> CrowdFundKey {
		let cf_key = create_cf_default(creator, amount);
		assert_ok!(CrowdFunding::open_contributions(RuntimeOrigin::root(), cf_key));
		let divided = amount / contributors.len() as u64;
		contributors.iter().for_each(|acc| {
			assert_ok!(CrowdFunding::contribute(RuntimeOrigin::signed(*acc), cf_key, divided));			
		});
		cf_key
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
		let a = CrowdFundCount::<Test>::get();
		create_cf_default(*ALICE, 100_000u64);
		let b = CrowdFundCount::<Test>::get();
		assert_eq!(b - 1, a);
		create_cf_default(*BOB, 100_000u64); 
		let c = CrowdFundCount::<Test>::get();
		assert_eq!(c - 1, b);
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
fn update_crowdfund_already_converted() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default_and_contribute(*ALICE, vec![*BOB, *CHARLIE], 100_000);
		let _ = CrowdFunding::approve_crowdfund_for_milestone_submission(RuntimeOrigin::root(), key).unwrap();
		assert_noop!(CrowdFunding::update_crowdfund(RuntimeOrigin::signed(*ALICE), key,
			None,
			None,
			None,
			None,
		), Error::<Test>::CrowdFundAlreadyConverted);
	});
}


#[test]
fn update_crowdfund_already_approved() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000);
		assert_ok!(CrowdFunding::open_contributions(RuntimeOrigin::root(), key));
		assert_noop!(CrowdFunding::update_crowdfund(RuntimeOrigin::signed(*ALICE), key,
			None,
			None,
			None,
			None,
		), Error::<Test>::CrowdFundAlreadyApproved);
	});
}

#[test]
fn update_crowdfund_already_cancelled() {
	new_test_ext().execute_with(|| {
		// todo: cancel extrinsic
	});
}

#[test]
fn update_crowdfund_not_initiator() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000);
		assert_noop!(CrowdFunding::update_crowdfund(RuntimeOrigin::signed(*BOB), key,
			None,
			None,
			None,
			None,
		), Error::<Test>::UserIsNotInitiator);
	});
}

#[test]
fn update_crowdfund_none_values_dont_mutate() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000);
		let cf_before = CrowdFunds::<Test>::get(key).expect("qed");
		assert_ok!(CrowdFunding::update_crowdfund(RuntimeOrigin::signed(*ALICE), key,
			None,
			None,
			None,
			None,
		));
		let cf = CrowdFunds::<Test>::get(key).expect("qed");
		assert_eq!(cf, cf_before);
	});
}

#[test]
fn update_crowdfund_actually_mutates() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000);
		let cf_before = CrowdFunds::<Test>::get(key).expect("qed");
		assert_ok!(CrowdFunding::update_crowdfund(RuntimeOrigin::signed(*ALICE), key,
			Some(get_milestones(20)),
			Some(50_000),
			Some(CurrencyId::KSM),
			Some(get_hash(69u8)),
		));
		
		let cf_after = CrowdFunds::<Test>::get(key).expect("qed");
		assert_ne!(cf_before, cf_after);
		assert_eq!(cf_after.milestones.len(), 20usize);
		assert_eq!(cf_after.required_funds, 50_000u64);
		assert_eq!(cf_after.currency_id, CurrencyId::KSM);
		assert_eq!(cf_after.agreement_hash, get_hash(69));
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
		assert_noop!(CrowdFunding::open_contributions(RuntimeOrigin::signed(*ALICE), key), BadOrigin);
	});
}

#[test]
fn open_contributions_approves_for_funding() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 100_000u64);
		assert_ok!(CrowdFunding::open_contributions(RuntimeOrigin::root(), key));
		let cf = CrowdFunds::<Test>::get(key).expect("oops");
		assert!(cf.approved_for_funding);
	});
}


#[test]
fn open_contributions_crowdfund_doesnt_exist_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(CrowdFunding::open_contributions(RuntimeOrigin::root(), Default::default()), Error::<Test>::CrowdFundDoesNotExist);
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
fn new_contribute_doesnt_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), 0, 100_000), Error::<Test>::CrowdFundDoesNotExist);
	});
}

#[test]
fn multiple_contributions_sum_contribution() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 1_000_000u64);
		let _ = CrowdFunding::open_contributions(RuntimeOrigin::root(), key).expect("should be fine.");

		// Contribute once, assert it was added.
		assert_ok!(CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, 100_000));
		let cf = CrowdFunds::<Test>::get(key).expect("qed");
		let contribution = cf.contributions.get(&BOB).expect("bob is cont. qed");
		assert_eq!(contribution.value, 100_000);

		// Contribute twice assert it sums.
		assert_ok!(CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, 50_000));
		let cf = CrowdFunds::<Test>::get(key).expect("just made it.");
		let contribution = cf.contributions.get(&BOB).expect("bob is cont. qed");
		assert_eq!(contribution.value, 150_000);

	});
}

#[test]
fn contribution_reserves_balance() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 1_000_000u64);
		let _ = CrowdFunding::open_contributions(RuntimeOrigin::root(), key).expect("should be fine.");
		let _ = CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, 100_000);
		let bob_reserved = <<Test as Config>::MultiCurrency as MultiReservableCurrency<AccountId>>::reserved_balance(CurrencyId::Native, &BOB);
		assert_eq!(bob_reserved, 100_000u64);

		let _ = CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, 50_000);
		let bob_reserved = <<Test as Config>::MultiCurrency as MultiReservableCurrency<AccountId>>::reserved_balance(CurrencyId::Native, &BOB);
		assert_eq!(bob_reserved, 150_000u64);
	});
}

#[test]
fn contribution_mutates_raised_funds() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 1_000_000u64);
		let _ = CrowdFunding::open_contributions(RuntimeOrigin::root(), key).expect("should be fine.");
		let _ = CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, 100_000);
		let _ = CrowdFunding::contribute(RuntimeOrigin::signed(*CHARLIE), key, 50_000);

		let cf = CrowdFunds::<Test>::get(key).expect("just created it;");
		assert_eq!(cf.raised_funds, 150_000);
		let _ = CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, 25_000);
		let cf = CrowdFunds::<Test>::get(key).expect("just created it;");
		assert_eq!(cf.raised_funds, 175_000);
	});
}

#[test]
fn ensure_initiator_works() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 1_000_000u64);
		assert!(CrowdFunding::ensure_initiator(*ALICE, key).is_ok());
		assert!(CrowdFunding::ensure_initiator(*BOB, key).is_err());
	});
}

#[test]
fn do_approve_not_authority() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default_and_contribute(*ALICE, vec![*BOB], 100_000);
		assert_noop!(CrowdFunding::approve_crowdfund_for_milestone_submission(RuntimeOrigin::signed(*ALICE), key), BadOrigin);
		assert_noop!(CrowdFunding::approve_crowdfund_for_milestone_submission(RuntimeOrigin::signed(*BOB), key), BadOrigin);
	});
}

#[test]
fn do_approve_raised_funds_not_reached() {
	new_test_ext().execute_with(|| {
		let amount: Balance = 1_000_000;
		let key = create_cf_default(*ALICE, amount);
		let _ = CrowdFunding::open_contributions(RuntimeOrigin::root(), key).expect("should be fine.");
		let _ = CrowdFunding::contribute(RuntimeOrigin::signed(*BOB), key, amount - 100);
		
		assert_noop!(CrowdFunding::approve_crowdfund_for_milestone_submission(RuntimeOrigin::root(), key), Error::<Test>::RequiredFundsNotReached);
	});
}

#[test]
fn do_approve_does_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(CrowdFunding::approve_crowdfund_for_milestone_submission(RuntimeOrigin::root(), 0), Error::<Test>::CrowdFundDoesNotExist);
	});
}

#[test]
fn on_initialize_removes_contribution_round() {
	new_test_ext().execute_with(|| {
		let key = create_cf_default(*ALICE, 1_000_000u64);
		let _ = CrowdFunding::open_contributions(RuntimeOrigin::root(), key).expect("should be fine.");
		let expiry_block: BlockNumber = frame_system::Pallet::<Test>::block_number() + <Test as Config>::RoundExpiry::get();
		run_to_block(expiry_block + 1);

		assert!(!CrowdFundsInRound::<Test>::contains_key(key, RoundType::ContributionRound));
		assert!(!RoundsExpiring::<Test>::contains_key(expiry_block));
	});

}

