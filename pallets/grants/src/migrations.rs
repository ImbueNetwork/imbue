
use crate::*;
pub use pallet::*;
use frame_support::{
    pallet_prelude::*,
    storage_alias,
    weights::Weight
};
use common_types::{CurrencyId, TreasuryOrigin};
use crate::mock::*;
use sp_runtime::traits::Zero;
use sp_core::H256;

type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

#[allow(unused)]
#[allow(dead_code)]
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
    pub type PendingGrants<T: Config> = StorageMap<Pallet<T>, Blake2_128, GrantId, GrantV0<T>, OptionQuery>;
}

#[allow(unused)]
#[allow(dead_code)]
mod v1 {
    use super::*;
    
    #[derive(Encode, Decode, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct GrantV1<T: Config> {
        pub milestones: BoundedPMilestones<T>,
        pub submitter: AccountIdOf<T>,
        pub approvers: BoundedApprovers<T>,
        pub created_on: BlockNumberFor<T>,
        pub is_cancelled: bool,
        pub is_converted: bool,
        pub currency_id: CurrencyId,
        pub amount_requested: BalanceOf<T>,
        pub treasury_origin: TreasuryOrigin,
        pub deposit_id: DepositIdOf<T>,
    }
    #[storage_alias]
    pub type PendingGrants<T: Config> = StorageMap<Pallet<T>, Blake2_128Concat, GrantId, GrantV1<T>, OptionQuery>;

    pub fn rococo_migrate_to_v1<T: Config>(weight: &mut Weight) {
        // This is only for rococo so just clear the lot, (there were only 4 at time of writing)
        let limit: u32 = 10;
        *weight += T::DbWeight::get().reads_writes(limit.into(), limit.into());
        let _ = v0::PendingGrants::<T>::clear(limit, None);
    }
}

#[allow(unused)]
#[allow(dead_code)]
mod v2 {
    use v1::*;
    use super::*;
    // The field in GrantV1 is_converted was removed.
    // There was no need for this field anymore as we just delete the grant or dont save it at all on conversion.
    pub fn migrate_is_converted<T: Config>(weight: &mut Weight) {
        crate::PendingGrants::<T>::translate(|key, grant: GrantV1<T>| {
            *weight += T::DbWeight::get().reads_writes(1, 1);
            let new = crate::Grant {
                milestones: grant.milestones,
                submitter: grant.submitter,
                approvers: grant.approvers,
                created_on: grant.created_on,
                is_cancelled: grant.is_cancelled,
                currency_id: grant.currency_id,
                amount_requested: grant.amount_requested,
                treasury_origin: grant.treasury_origin,
                deposit_id: grant.deposit_id,
            };
            Some(new)
        })
    }

    #[test]
    fn migrate_converted() {
        new_test_ext().execute_with(|| {

            let grant = v1::GrantV1 {
                milestones: tests::get_milestones(10),
                submitter: *ALICE,
                approvers: tests::get_approvers(10),
                created_on: frame_system::Pallet::<Test>::block_number(),
                is_cancelled: false,
                is_converted: false,
                currency_id: CurrencyId::Native,
                amount_requested: 100_000_000_000,
                treasury_origin: TreasuryOrigin::Kusama,
                deposit_id: u64::MAX,
            };
            v1::PendingGrants::<Test>::insert(<H256 as Default>::default(), grant);
            let mut weight: Weight = Zero::zero();
            migrate_is_converted::<Test>(&mut weight);
            let v1: Option<v1::GrantV1<Test>> = v1::PendingGrants::<Test>::get(<H256 as Default>::default());
            let v2 = crate::PendingGrants::<Test>::get(<H256 as Default>::default());
            assert!(v1.is_none());
            assert!(v2.is_some());
        })
    }
}
