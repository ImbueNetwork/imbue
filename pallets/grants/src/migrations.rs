
use crate::*;
pub use pallet::*;
use frame_support::{
    pallet_prelude::*,
    storage_alias,
    weights::Weight
};
use common_types::{CurrencyId, TreasuryOrigin};

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
    pub type PendingGrantsV0<T: Config> = StorageMap<Pallet<T>, Blake2_128, GrantId, GrantV0<T>, OptionQuery>;
}

#[allow(unused)]
#[allow(dead_code)]
mod v1 {
    use super::*;
    pub fn rococo_migrate_to_v1<T: Config>(weight: &mut Weight) {
        // This is only for rococo so just clear the lot, (there were only 4 at time of writing)
        let limit: u32 = 10;
        *weight += T::DbWeight::get().reads_writes(limit.into(), limit.into());
        let _ = v0::PendingGrantsV0::<T>::clear(limit, None);
    }
}
