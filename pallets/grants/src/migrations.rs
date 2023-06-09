#[allow(unused)]
use crate::*;
pub use pallet::*;
use frame_support::{
    pallet_prelude::*,
    storage_alias,
    traits::Get,
    weights::Weight
};
use common_types::{CurrencyId, TreasuryOrigin};
use sp_std::convert::TryInto;
use pallet_proposals::ProposedMilestone;
use sp_core::H256;
use sp_arithmetic::Percent;
use sp_std::vec::Vec;

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
    pub type PendingGrantsV0<T: Config> = StorageMap<Pallet<T>, Blake2_128, GrantId, GrantV0<T>, OptionQuery>;
}

mod v1 {
    use super::*;
    pub fn rococo_migrate_to_v1<T: Config>(weight: &mut Weight) {
        // This is only for rococo so just clear the lot, (there were only 4 at time of writing)
        v0::PendingGrantsV0::<T>::clear(10, None);
    }
}
