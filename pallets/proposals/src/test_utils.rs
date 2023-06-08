use crate::Config;
use crate::Pallet as Proposals;
use crate::{
    AccountIdOf, BalanceOf, Contribution, ContributionsFor, Milestone, MilestoneKey, Project,
    ProjectKey, ProposedMilestone,
};
use common_types::{CurrencyId, FundingType};
use frame_benchmarking::{account, Vec};
use frame_support::{assert_ok, traits::Hooks};
use frame_system::EventRecord;
use orml_traits::MultiCurrency;
use sp_arithmetic::{per_things::Percent, traits::Zero};
use sp_core::{Get, H256};
use sp_runtime::Saturating;
use sp_std::{convert::TryInto, collections::btree_map::BTreeMap};
use pallet_deposits::traits::DepositHandler;

pub fn run_to_block<T: Config>(n: T::BlockNumber) {
    loop {
        let mut block = frame_system::Pallet::<T>::block_number();
        if block >= n {
            break;
        }
        block = block.saturating_add(1u32.into());
        frame_system::Pallet::<T>::set_block_number(block);
        frame_system::Pallet::<T>::on_initialize(block);
        Proposals::<T>::on_initialize(block);
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
        contributions.try_insert(account.clone(), contribution).expect("bound should be ensured");
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

    let deposit_id = <T as Config>::DepositHandler::take_deposit(beneficiary.clone(), <T as Config>::ProjectStorageItem::get(), CurrencyId::Native);
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
        milestones: milestones.try_into().expect("too many milestones"),
        contributions: contributions.try_into().expect("too many contributions"),
        currency_id,
        withdrawn_funds: 0u32.into(),
        raised_funds,
        initiator: beneficiary.clone(),
        created_on: frame_system::Pallet::<T>::block_number(),
        cancelled: false,
        agreement_hash,
        funding_type: FundingType::Brief,
        deposit_id,
    };

    crate::Projects::<T>::insert(project_key, project);
    crate::ProjectCount::<T>::put(project_key);

    project_key
}

pub fn create_funded_user<T: Config>(
    seed: &'static str,
    n: u32,
    balance_factor: u32,
) -> T::AccountId {
    let user = account(seed, n, 0);
    let balance: BalanceOf<T> = balance_factor.into();
    assert_ok!(<T::MultiCurrency as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::deposit(CurrencyId::Native, &user, balance,));
    user
}

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent)
where
    <T as frame_system::Config>::AccountId: AsRef<[u8]>,
{
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}
