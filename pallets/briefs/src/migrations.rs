use crate::mock::*;
#[allow(unused)]
use crate::*;
use common_types::CurrencyId;
use frame_support::{pallet_prelude::*, storage_alias, traits::Get, weights::Weight};
pub use pallet::*;
use pallet_projects::ProposedMilestone;
use sp_arithmetic::Percent;
use sp_core::H256;
use sp_std::convert::TryInto;

type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

mod v0 {
    use super::*;

    #[derive(Encode, Decode, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct BriefDataV0<T: Config> {
        pub(crate) brief_owners: BoundedBriefOwners<T>,
        pub(crate) budget: BalanceOf<T>,
        pub(crate) currency_id: CurrencyId,
        pub(crate) created_at: BlockNumberFor<T>,
        pub(crate) applicant: AccountIdOf<T>,
        pub(crate) milestones:
            BoundedVec<ProposedMilestoneV0, <T as Config>::MaxMilestonesPerBrief>,
    }

    #[derive(Encode, Decode, Debug, MaxEncodedLen, TypeInfo)]
    pub struct ProposedMilestoneV0 {
        pub percentage_to_unlock: u32,
    }

    #[storage_alias]
    pub type Briefs<T: Config> =
        CountedStorageMap<Pallet<T>, Blake2_128Concat, BriefHash, v0::BriefDataV0<T>, OptionQuery>;
}

// Migrate the proposed milestones to use Percent over a u32.
// Add a deposit id to BriefData.
// Should be run with pallet_projects::migrations::v3
mod v1 {
    use super::*;
    pub fn migrate_to_v1<T: Config>(weight: &mut Weight) {
        crate::Briefs::<T>::translate(|_, brief: v0::BriefDataV0<T>| {
            *weight += T::DbWeight::get().reads_writes(2, 1);
            let maybe_milestones: Result<BoundedProposedMilestones<T>, _> = brief
                .milestones
                .iter()
                .map(|ms| {
                    let convert: Result<u8, _> = ms.percentage_to_unlock.try_into();
                    if let Ok(n) = convert {
                        Some(ProposedMilestone {
                            percentage_to_unlock: Percent::from_percent(n),
                        })
                    } else {
                        None
                    }
                })
                .flatten()
                .collect::<Vec<ProposedMilestone>>()
                .try_into();

            if let Ok(milestones) = maybe_milestones {
                if milestones.len() != brief.milestones.len() {
                    return None;
                }
                Some(crate::BriefData {
                    brief_owners: brief.brief_owners,
                    budget: brief.budget,
                    currency_id: brief.currency_id,
                    created_at: brief.created_at,
                    applicant: brief.applicant,
                    milestones,
                    // A deposit_id of U32::Max is skipped and not returned.
                    deposit_id: u32::MAX.into(),
                })
            } else {
                None
            }
        })
    }

    #[test]
    fn migrate_v0_to_v1() {
        build_test_externality().execute_with(|| {
            let milestones: BoundedVec<
                v0::ProposedMilestoneV0,
                <Test as Config>::MaxMilestonesPerBrief,
            > = vec![
                v0::ProposedMilestoneV0 {
                    percentage_to_unlock: 80u32,
                },
                v0::ProposedMilestoneV0 {
                    percentage_to_unlock: 20u32,
                },
            ]
            .try_into()
            .expect("2 should be lower than bound");

            let old_brief = v0::BriefDataV0 {
                brief_owners: vec![*ALICE, *BOB]
                    .try_into()
                    .expect("2 should be lower than bound"),
                budget: 100_000u64,
                currency_id: CurrencyId::Native,
                created_at: frame_system::Pallet::<Test>::block_number(),
                applicant: *CHARLIE,
                milestones,
            };
            let key: H256 = [1; 32].into();
            v0::Briefs::<Test>::insert(key, &old_brief);
            let mut weight: Weight = Default::default();
            v1::migrate_to_v1::<Test>(&mut weight);

            let new_brief = crate::Briefs::<Test>::get(key).expect("should exist.");
            assert_eq!(new_brief.deposit_id, u32::MAX as u64);
            assert_eq!(
                new_brief.milestones[0].percentage_to_unlock,
                Percent::from_percent(old_brief.milestones[0].percentage_to_unlock as u8)
            );
            assert_eq!(
                new_brief.milestones[1].percentage_to_unlock,
                Percent::from_percent(old_brief.milestones[1].percentage_to_unlock as u8)
            );
        })
    }
}
