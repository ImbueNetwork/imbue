use crate::*;
pub use pallet::*;
use common_types::{CurrencyId, TreasuryOrigin};
use frame_support::{pallet_prelude::*, storage_alias, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;

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
    pub type PendingGrants<T: Config> =
        StorageMap<Pallet<T>, Blake2_128, GrantId, GrantV0<T>, OptionQuery>;
}

#[allow(unused)]
#[allow(dead_code)]
mod v1 {
    use super::*;

    #[derive(Encode, Decode, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Grant<T: Config> {
        pub milestones: BoundedPMilestones<T>,
        pub submitter: AccountIdOf<T>,
        pub approvers: BoundedApprovers<T>,
        pub created_on: BlockNumberFor<T>,
        pub is_cancelled: bool,
        pub is_converted: bool,
        pub currency_id: CurrencyId,
        pub amount_requested: BalanceOf<T>,
        pub treasury_origin: TreasuryOrigin,
        pub deposit_id: u64,
    }

    #[storage_alias]
    pub type PendingGrants<T: Config> =
        StorageMap<Pallet<T>, Blake2_128Concat, GrantId, v1::Grant<T>, OptionQuery>;

    pub fn rococo_migrate_to_v1<T: Config>(weight: &mut Weight) {
        // This is only for rococo so just clear the lot, (there were only 4 at time of writing)
        if v3::StorageVersion::<T>::get() == v3::Release::V0 {
            let limit: u32 = 10;
            *weight += T::DbWeight::get().reads_writes(limit.into(), limit.into());
            let _ = v0::PendingGrants::<T>::clear(limit, None);
            v3::StorageVersion::<T>::put(v3::Release::V1);
        }
    }
}

#[allow(unused)]
#[allow(dead_code)]
pub(crate) mod v2 {
    use super::*;
    // We are not storing pending grants anymore and grants are going directly into projects.
    pub fn migrate_to_v2<T: Config>(weight: &mut Weight, limit: u32) {
        if v3::StorageVersion::<T>::get() == v3::Release::V1 {
            *weight += T::DbWeight::get().reads_writes(limit.into(), limit.into());
            let _ = v1::PendingGrants::<T>::clear(limit, None);
            v3::StorageVersion::<T>::put(v3::Release::V2);
        }
    }
}




pub mod v3 {
        use super::*;
    
        #[storage_alias]
        pub type StorageVersion<T: Config> = StorageValue<Pallet<T>, Release, ValueQuery>;
    
        #[derive(Encode, Decode, TypeInfo, PartialEq, MaxEncodedLen, Default)]
        #[repr(u32)]
        pub enum Release {
            V0,
            V1,
            #[default]
            V2,
        }

        pub struct MigrateToV3<T: Config>(T);
        impl<T: Config> OnRuntimeUpgrade<T> for MigrateToV3<T> {
            #[cfg(feature = "try-runtime")]
            fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
                frame_support::ensure!(
                    StorageVersion::<T>::get() == v3::Release::V2,
                    "V2 is required before running V3"
                );
                
                Ok(<Vec<u8> as Default>::default())
            }
            
            fn on_runtime_upgrade() -> Weight {
                let current = Pallet::<T>::current_storage_version();
                let onchain = StorageVersion::<T>::get();
    
                if current == 3 && onchain == v3::Release::V2 {
                    StorageVersion::<T>::kill();
                    current.put::<Pallet<T>>();
    
                    log::warn!("v2 has been successfully applied");
                    T::DbWeight::get().reads_writes(2, 1)
                } else {
                    log::warn!("Skipping v2, should be removed");
                    T::DbWeight::get().reads(1)
                }
            }
    
            #[cfg(feature = "try-runtime")]
            fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
                frame_support::ensure!(
                    Pallet::<T>::current_storage_version() == 3,
                    "v2 has not been applied"
                );
    
                ensure!(
                    !StorageVersion::<T>::exists(),
                    "old storage version has not been removed."
                );
    
                Ok(())
            }
        }
}
