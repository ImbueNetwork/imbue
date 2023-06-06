#[allow(unused)]
use crate::*;
pub use pallet::*;
use crate::mock::*;
use frame_support::{
    pallet_prelude::*,
    storage_alias, 
    traits::Get, 
    weights::Weight
};
use common_types::CurrencyId;
use sp_std::convert::TryInto;
use pallet_proposals::ProposedMilestone;
use sp_core::H256;
use sp_arithmetic::Percent;

type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

mod v0 {
    use super::*;
    #[derive(Encode, Decode, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct GrantV0<T: Config> {
        pub milestones: BoundedPMilestones<T>,
        pub submitter: AccountIdOf<T>,
        pub approvers: BoundedApprovers<T>,
        pub created_on: BlockNumberFor<T>,
        pub is_cancelled: bool,
        pub is_converted: bool,
        pub currency_id: CurrencyId,
        pub amount_requested: BalanceOf<T>,
        pub treasury_origin: TreasuryOrigin,
    }

    #[derive(Encode, Decode, Debug, MaxEncodedLen, TypeInfo)]
    pub struct ProposedMilestoneV0 {
        pub percentage_to_unlock: u32,
    }

    #[storage_alias]
    pub type PendingGrantsV0<T: Config> = StorageMap<_, Blake2_128, GrantId, GrantV0<T>, OptionQuery>;
}

// Migrate the proposed milestones to use Percent over a u32.
// Add a deposit id to Grant.
// Should be run with pallet_proposals::migrations::v3
mod v1 {
    use super::*;
    pub fn migrate_to_v1<T: Config>(weight: &mut Weight) {
        crate::PendingGrants::migrate(|_key, g| {
            *weight += T::DbWeight::get().reads_writes(2, 1);
            let maybe_milestones: Result<BoundedPMilestones<T>, _> = g.milestones.iter().map(|ms| {
                let convert: Result<u8, _> = ms.percentage_to_unlock.try_into();
                if let Ok(n) = convert {
                    Some(ProposedMilestone {
                        percentage_to_unlock: Percent::from_percent(n)
                    })           
                } else {
                    None;
                }
            }).flatten().collect::<Vec<ProposedMilestone>>().try_into();
            if let Ok(milestones) = maybe_milestones {
                if milestones.len() != g.milestones.len() {
                    return None
                }
                Some(crate::Grant {
                    milestones,
                    submitter: g.submitter,
                    approvers: g.approvers,
                    created_on: g.created_on,
                    is_cancelled: g.is_cancelled,
                    is_converted: g.is_converted,
                    currency_id: g.currency_id,
                    amount_requested: g.amount_requested,
                    // u32 max are ignored in the pallet-deposit conversion for backwards compatibility.
                    treasury_origin: u32::MAX.into(),
                })
            } else {
                None
            }
        })
    }

    #[test]
    fn migrate_v0_to_v1() {
        build_test_externality().execute_with(|| {
            
            let milestones: BoundedVec<v0::ProposedMilestoneV0, <Test as Config>::MaxMilestonesPerBrief> = vec![
                v0::ProposedMilestoneV0 {
                    percentage_to_unlock: 80u32,
                },
                v0::ProposedMilestoneV0 {
                    percentage_to_unlock: 20u32,
                } 
            ].try_into().expect("2 should be lower than bound");

            let old_grant = v0::GrantV0 {
                milestones,
                submitter: *ALICE,
                approvers: vec![*BOB, *CHARLIE].try_into().expect("2 should be lower than bound"),
                created_on: frame_system::Pallet::<Test>::block_number(),
                is_cancelled: false,
                is_converted: false,
                currency_id: CurrencyId::Native,
                amount_requested: 100_000u64,
                treasury_origin: TreasuryOrigin::Imbue,
            }
            let grant_id: H256 = [2; 32].into();
            v0::PendingGrantsV0::insert(grant_id, old_grant);
            let mut weight: Weight = Default::default();
            v1::migrate_to_v1(&mut weight);
            let new_grant = crate::Grants::<Test>::get(grant_id).expect("should exist");

            assert!(!v0::PendingGrantsV0::contains_key(grant_id));
            assert_eq(new_grant.deposit_id, u32::MAX.into());
            assert_eq!(new_grant.milestones[0].percentage_to_unlock, Percent::from_percent(old_grant.milestones[0].percentage_to_unlock as u8));
            assert_eq!(new_grant.milestones[1].percentage_to_unlock, Percent::from_percent(old_grant.milestones[1].percentage_to_unlock as u8));

        })
    }
}
