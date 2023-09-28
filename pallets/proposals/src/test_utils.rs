use crate::Config;
use crate::Pallet as Proposals;
use crate::*;
use common_types::{CurrencyId, FundingType};
#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{account, Vec};
use frame_support::{assert_ok, traits::Hooks, BoundedVec};
use frame_system::EventRecord;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use pallet_deposits::traits::DepositHandler;
use sp_arithmetic::per_things::Percent;
use sp_core::{Get, H256};
use sp_runtime::Saturating;
use sp_runtime::{DispatchError, SaturatedConversion};
use sp_std::{collections::btree_map::BTreeMap, convert::TryInto};

#[allow(dead_code)]
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

// Using the FundingPath::TakeFromReserved create a project for testing funded milestones
// This will be called in the majority of test cases.
// IntoProposal assumes that funds have been reserved before calling it.
pub fn create_and_fund_project<T: Config>(
    beneficiary: AccountIdOf<T>,
    contributions: ContributionsFor<T>,
    proposed_milestones: Vec<ProposedMilestone>,
    currency_id: CurrencyId,
) -> Result<ProjectKey, DispatchError> {
    contributions.iter().for_each(|(acc, c)| {
        <T as Config>::MultiCurrency::reserve(currency_id, acc, c.value).unwrap();
    });
    let agreement_hash: H256 = Default::default();
    let refund_locations = <Proposals<T> as IntoProposal<
        AccountIdOf<T>,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >>::convert_contributions_to_refund_locations(
        &contributions.clone().into_inner()
    );

    // Reserve the assets from the contributors used.
    <Proposals<T> as IntoProposal<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>>::convert_to_proposal(
        currency_id,
        contributions.into_inner(),
        agreement_hash,
        beneficiary,
        proposed_milestones,
        refund_locations,
        Vec::new(),
        FundingPath::TakeFromReserved,
    )?;

    Ok(ProjectCount::<T>::get())
}

// For testing grants and errors pre funding
pub fn create_project_awaiting_funding<T: Config>(
    beneficiary: AccountIdOf<T>,
    contributions: ContributionsFor<T>,
    proposed_milestones: Vec<ProposedMilestone>,
    currency_id: CurrencyId,
    treasury_account: MultiLocation,
) -> Result<ProjectKey, DispatchError> {
    let agreement_hash: H256 = Default::default();
    // Reserve the assets from the contributors used.
    <Proposals<T> as IntoProposal<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>>::convert_to_proposal(
        currency_id,
        contributions.into_inner(),
        agreement_hash,
        beneficiary,
        proposed_milestones,
        vec![(Locality::Foreign(treasury_account), Percent::from_parts(100u8))],
        Vec::new(),
        FundingPath::WaitForFunding,
    )?;

    Ok(ProjectCount::<T>::get())
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
