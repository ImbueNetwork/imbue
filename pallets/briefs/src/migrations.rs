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

//TODO: ProposedMilestones from u8 to percent.
    #[derive(Encode, Decode, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct BriefDataV0<T: Config> {
        pub(crate) brief_owners: BoundedBriefOwners<T>,
        pub(crate) budget: BalanceOf<T>,
        pub(crate) currency_id: CurrencyId,
        pub(crate) created_at: BlockNumberFor<T>,
        pub(crate) applicant: AccountIdOf<T>,
        pub(crate) milestones: BoundedProposedMilestones<T>,
    }
    #[storage_alias]
    pub type Briefs<T: Config> =
        CountedStorageMap<Pallet<T>, Blake2_128Concat, BriefHash, v0::BriefDataV0<T>, OptionQuery>;
}

mod v1 {
    use super::*;
    pub fn migrate_to_v1<T: Config>(weight: &mut Weight) {
        crate::Briefs::<T>::translate(|key, brief: v0::BriefDataV0<T>| {
            *weight += T::DbWeight::get().reads_writes(1, 1);
            Some(crate::BriefData {
                brief_owners: brief.brief_owners,
                budget: brief.budget,
                currency_id: brief.currency_id,
                created_at: brief.created_at,
                applicant: brief.applicant,
                milestones: brief.milestones,
                // A deposit_id of U32::Max is skipped and not returned. 
                deposit_id: u32::MAX.into(),
            })
        })
    }

    #[test]
    fn migrate_v0_to_v1() {
        build_test_externality().execute_with(|| {
            let milestones: BoundedProposedMilestones<Test> = vec![
                ProposedMilestone {
                    percentage_to_unlock: Percent::from_percent(60u8)
                },
                ProposedMilestone {
                    percentage_to_unlock: Percent::from_percent(40u8)
                } 
            ].try_into().expect("2 should be lower than bound");

            let old_brief = v0::BriefDataV0 {
                brief_owners: vec![*ALICE, *BOB].try_into().expect("2 should be lower than bound"),
                budget: 100_000u64,
                currency_id: CurrencyId::Native,
                created_at: frame_system::Pallet::<Test>::block_number(),
                applicant: *CHARLIE,
                milestones,
            };            
            let key: H256 = [1; 32].into();
            v0::Briefs::<Test>::insert(key, old_brief);
            let mut weight: Weight = Default::default();
            v1::migrate_to_v1::<Test>(&mut weight);

            let new_brief = crate::Briefs::<Test>::get(key).expect("should exist.");
            assert_eq!(new_brief.deposit_id, u32::MAX as u64);
        })
    }
}
