#![cfg(feature = "runtime-benchmarks")]
#![allow(unused_imports)]

use super::*;
use crate::test_utils::gen_grant_id;
use crate::Pallet as Grants;
use crate::{BoundedApprovers, BoundedPMilestones, Config};
use common_types::{CurrencyId, TreasuryOrigin};
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::Get};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use pallet_proposals::ProposedMilestone;
use sp_arithmetic::per_things::Percent;
use sp_runtime::SaturatedConversion;
use sp_std::{convert::TryInto, str, vec::Vec};

const SEED: u32 = 0;

#[benchmarks]
mod benchmarks {
    use super::*;
    // This assumes that the weight of the benchmark increases at a linear rate depending
    // on approvers and milestones.
    #[benchmark]
    fn create_and_convert() {
        let submitter: T::AccountId = create_account_id::<T>("submitter", 1);
        let grant_id = gen_grant_id(1);
        let approvers = get_approvers::<T>(<T as Config>::MaxApprovers::get());
        let milestones = get_milestones::<T>(<T as Config>::MaxMilestonesPerGrant::get());
        let amount_requested = 1_000_000u32.into();

        #[extrinsic_call]
        create_and_convert(
            RawOrigin::Signed(submitter),
            milestones,
            approvers,
            CurrencyId::Native,
            amount_requested,
            TreasuryOrigin::Kusama,
            grant_id,
        );
    }
    impl_benchmark_test_suite!(Grants, crate::mock::new_test_ext(), crate::mock::Test);
}

fn get_approvers<T: Config>(n: u32) -> BoundedApprovers<T> {
    (0..n)
        .map(|i| create_account_id::<T>("brief_owner", i))
        .collect::<Vec<T::AccountId>>()
        .try_into()
        .expect("qed")
}

fn get_milestones<T: Config>(n: u32) -> BoundedPMilestones<T> {
    (0..n)
        .map(|_| ProposedMilestone {
            percentage_to_unlock: Percent::from_percent((100 / n) as u8),
        })
        .collect::<Vec<ProposedMilestone>>()
        .try_into()
        .expect("qed")
}

fn create_account_id<T: Config>(suri: &'static str, n: u32) -> T::AccountId {
    let user = account(suri, n, SEED);
    let initial_balance = 10_000_000_000_000_000u128;
    assert_ok!(T::RMultiCurrency::deposit(
        CurrencyId::Native,
        &user,
        initial_balance.saturated_into()
    ));
    user
}
