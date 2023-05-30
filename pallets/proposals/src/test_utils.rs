use crate::mock::System;
use crate::Config;
use crate::Pallet as Proposals;
use crate::{
    AccountIdOf, BalanceOf, Contribution, ContributionsFor, Milestone, MilestoneKey, Project,
    ProjectKey, ProposedMilestone,
};
use common_types::{CurrencyId, FundingType};
use frame_support::{assert_ok, traits::Hooks};
use orml_traits::MultiCurrency;
use sp_arithmetic::per_things::Percent;
use sp_core::H256;
use sp_runtime::Saturating;
use sp_std::collections::btree_map::BTreeMap;

pub fn run_to_block<T: Config>(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Proposals::<T>::on_initialize((System::block_number() as u32).into());
    }
}

pub fn get_contributions<T: Config>(
    accounts: Vec<AccountIdOf<T>>,
    total_amount: u32,
) -> ContributionsFor<T> {
    let value: BalanceOf<T> = (total_amount / accounts.len() as u32).into();
    let timestamp = frame_system::Pallet::<T>::block_number();
    let mut contributions: ContributionsFor<T> = Default::default();

    accounts.iter().for_each(|account| {
        let contribution = Contribution { value, timestamp };
        contributions.insert(account.clone(), contribution);
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

/// Create a project for test purposes, this will not test the paths coming into this pallet via
/// the IntoProposal trait.
pub fn create_project<T: Config>(
    beneficiary: AccountIdOf<T>,
    contributions: ContributionsFor<T>,
    proposed_milestones: Vec<ProposedMilestone>,
    currency_id: CurrencyId,
) -> ProjectKey {
    let agreement_hash: H256 = Default::default();

    let project_key = crate::ProjectCount::<T>::get().saturating_add(1);

    let mut raised_funds: BalanceOf<T> = 0u32.into();
    let project_account_id = Proposals::<T>::project_account_id(project_key);

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

    for ms in proposed_milestones {
        let milestone = Milestone {
            project_key,
            milestone_key,
            percentage_to_unlock: ms.percentage_to_unlock,
            is_approved: false,
        };
        milestones.insert(milestone_key, milestone);
        milestone_key = milestone_key.saturating_add(1);
    }

    let project = Project {
        milestones,
        contributions,
        currency_id,
        withdrawn_funds: 0u32.into(),
        raised_funds,
        initiator: beneficiary.clone(),
        created_on: frame_system::Pallet::<T>::block_number(),
        cancelled: false,
        agreement_hash,
        funding_type: FundingType::Brief,
    };

    crate::Projects::<T>::insert(project_key, project);
    crate::ProjectCount::<T>::put(project_key);

    project_key
}
