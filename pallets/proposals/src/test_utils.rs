use crate::*;
use common_types::{CurrencyId, FundingType};
#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::account;
use frame_support::assert_ok;
use frame_system::EventRecord;
use orml_traits::MultiCurrency;
use pallet_deposits::traits::DepositHandler;
use sp_arithmetic::per_things::Percent;
use sp_core::{Get, H256};
use sp_runtime::SaturatedConversion;
use sp_runtime::Saturating;
#[cfg(feature = "runtime-benchmarks")]
use sp_std::vec::Vec;
use sp_std::{collections::btree_map::BTreeMap, convert::TryInto};

pub fn get_contributions<T: Config>(
    accounts: Vec<AccountIdOf<T>>,
    contribution: u128,
) -> ContributionsFor<T> {
    let value: BalanceOf<T> = contribution.saturated_into();
    let timestamp = frame_system::Pallet::<T>::block_number();
    let mut contributions: ContributionsFor<T> = Default::default();

    accounts.iter().for_each(|account| {
        let contribution = Contribution { value, timestamp };
        contributions
            .try_insert(account.clone(), contribution)
            .expect("bound should be ensured");
    });
    contributions
}

pub fn get_milestones(n: u8) -> Vec<ProposedMilestone> {
    (0..n)
        .map(|_| ProposedMilestone {
            percentage_to_unlock: Percent::from_percent(100u8 / n),
        })
        .collect::<Vec<ProposedMilestone>>()
}

#[cfg(feature = "runtime-benchmarks")]
pub fn get_max_milestones<T: Config>() -> Vec<ProposedMilestone> {
    get_milestones(<T as Config>::MaxMilestonesPerProject::get() as u8)
}

/// Create a project for test purposes, this will not test the paths coming into this pallet via
/// the IntoProposal trait.
pub fn create_project<T: Config>(
    beneficiary: AccountIdOf<T>,
    contributions: ContributionsFor<T>,
    proposed_milestones: Vec<ProposedMilestone>,
    currency_id: CurrencyId,
) -> ProjectKey {
    let deposit_id = <T as Config>::DepositHandler::take_deposit(
        beneficiary.clone(),
        <T as Config>::ProjectStorageItem::get(),
        CurrencyId::Native,
    )
    .expect("this should work");
    let agreement_hash: H256 = Default::default();

    let project_key = crate::ProjectCount::<T>::get().saturating_add(1);

    let mut raised_funds: BalanceOf<T> = 0u32.into();
    let project_account_id = crate::Pallet::<T>::project_account_id(project_key);

    for (account, contribution) in contributions.iter() {
        let amount = contribution.value;
        assert_ok!(<T as crate::Config>::MultiCurrency::transfer(
            currency_id,
            account,
            &project_account_id,
            amount
        ));
        raised_funds = raised_funds.saturating_add(amount);
    }

    let mut milestone_key: u32 = 0;
    let mut milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();
    let mut bounded_milestone_keys: BoundedVec<MilestoneKey, T::MaxMilestonesPerProject> =
        BoundedVec::new();

    for ms in proposed_milestones {
        let milestone = Milestone {
            project_key,
            milestone_key,
            percentage_to_unlock: ms.percentage_to_unlock,
            is_approved: false,
            withdrawn: false,
        };
        milestones.insert(milestone_key, milestone);
        let _ = bounded_milestone_keys.try_push(milestone_key);
        milestone_key = milestone_key.saturating_add(1);
    }

    let individual_votes = ImmutableIndividualVotes::new(bounded_milestone_keys);
    IndividualVoteStore::<T>::insert(project_key, individual_votes);

    let project = Project {
        milestones: milestones.try_into().expect("too many milestones"),
        contributions,
        currency_id,
        withdrawn_funds: 0u32.into(),
        raised_funds,
        initiator: beneficiary,
        created_on: frame_system::Pallet::<T>::block_number(),
        cancelled: false,
        agreement_hash,
        funding_type: FundingType::Brief,
        deposit_id,
        payment_address: [0;20],
    };

    crate::Projects::<T>::insert(project_key, project);
    crate::ProjectCount::<T>::put(project_key);

    project_key
}

#[cfg(feature = "runtime-benchmarks")]
pub fn create_funded_user<T: Config>(
    seed: &'static str,
    n: u32,
    balance_factor: u128,
) -> T::AccountId {
    let user = account(seed, n, 0);
    assert_ok!(<T::MultiCurrency as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::deposit(
        CurrencyId::Native, &user, balance_factor.saturated_into()
    ));
    user
}

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}
