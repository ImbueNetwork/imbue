use crate::*;
use common_types::CurrencyId;
use frame_support::{assert_ok, BoundedVec};
use frame_system::EventRecord;
use orml_traits::{MultiCurrency, MultiReservableCurrency};

use pallet_disputes::traits::DisputeHooks;
use sp_arithmetic::per_things::Percent;
use sp_core::{Get, H256};

use sp_runtime::{DispatchError, SaturatedConversion};
use sp_std::convert::TryInto;

#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::account;
#[cfg(feature = "runtime-benchmarks")]
use sp_std::vec::Vec;

// pub fn run_to_block(n: BlockNumber) {
//     while System::block_number() < n {
//         Tokens::on_finalize(System::block_number());
//         System::on_finalize(System::block_number());
//         Proposals::on_finalize(System::block_number());
//         System::set_block_number(System::block_number() + 1);
//         Tokens::on_initialize(System::block_number());
//         System::on_initialize(System::block_number());
//         Proposals::on_initialize(System::block_number());
//     }
// }

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
    let refund_locations = <crate::Pallet<T> as IntoProposal<
        AccountIdOf<T>,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >>::convert_contributions_to_refund_locations(&contributions);

    // Reserve the assets from the contributors used.
    <crate::Pallet<T> as IntoProposal<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>>::convert_to_proposal(
        currency_id,
        contributions,
        agreement_hash,
        beneficiary,
        proposed_milestones.try_into().map_err(|_|Error::<T>::TooManyMilestones)?,
        refund_locations,
        BoundedVec::new(),
        FundingPath::TakeFromReserved,
    )?;

    Ok(ProjectCount::<T>::get())
}

// For testing grants and errors pre funding
// TODO: tests for these!
pub fn _create_project_awaiting_funding<T: Config>(
    beneficiary: AccountIdOf<T>,
    contributions: ContributionsFor<T>,
    proposed_milestones: Vec<ProposedMilestone>,
    currency_id: CurrencyId,
    treasury_account: MultiLocation,
) -> Result<ProjectKey, DispatchError> {
    let agreement_hash: H256 = Default::default();
    // Reserve the assets from the contributors used.
    <crate::Pallet<T> as IntoProposal<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>>::convert_to_proposal(
        currency_id,
        contributions,
        agreement_hash,
        beneficiary,
        proposed_milestones.try_into().map_err(|_|Error::<T>::TooManyMilestones)?,
        vec![(Locality::Foreign(treasury_account), Percent::from_parts(100u8))].try_into().map_err(|_|Error::<T>::TooManyRefundLocations)?,
        BoundedVec::new(),
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
    assert_ok!(
        <T::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::deposit(
            CurrencyId::Native,
            &user,
            balance_factor.saturated_into()
        )
    );
    user
}

/// Manually call the hook OnDisputeCompleteWith a predefined result for testing>
pub fn complete_dispute<T: Config>(
    project_key: ProjectKey,
    milestone_keys: Vec<MilestoneKey>,
    result: pallet_disputes::DisputeResult,
) -> crate::Weight {
    <crate::Pallet<T>>::on_dispute_complete(project_key, milestone_keys, result)
}

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}
