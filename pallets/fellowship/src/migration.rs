use sp_std::{vec, vec::Vec, str::FromStr, fmt::Debug};
use frame_support::traits::OnRuntimeUpgrade;
use frame_support::{*, pallet_prelude::*, dispatch::EncodeLike};
use sp_runtime::AccountId32;
use crate::{*, traits::*};

mod v0 {
    use super::*;

    struct MigrateInitial<T: crate::Config>(T);
    impl<T: Config> MigrateInitial<T> 
    where 
    <T as frame_system::Config>::AccountId: FromStr,
    <<T as frame_system::Config>::AccountId as FromStr>::Err : Debug

    {
        fn insert_initial_fellows(weight: &mut Weight) {
            let initial_fellows: Vec<(<T as frame_system::Config>::AccountId, crate::Role, crate::Rank)> = vec![
                // EARNEST
                (<AccountIdOf<T> as FromStr>::from_str("5Da1Fna8wvgQNmCFPhcRGR9oxmhyPd7MNhPZADq2X6GiKkkr").unwrap(), Role::Freelancer, 10),
                // ME
                (<AccountIdOf<T> as FromStr>::from_str("5DCzKK5EZvY77vxxWXeip7sp17TqB7sk7Fj1hXes7Bo6B5Eq").unwrap(), Role::Freelancer, 10),
                // BEA
                (<AccountIdOf<T> as FromStr>::from_str("5DU2hcQnEmrSXCDUnjiwNX3A1uTf26ACpgs4KUFpsLJqAnjd").unwrap(), Role::Freelancer, 10),
            ];
            for (acc, role, rank) in initial_fellows.into_iter() {
                <Pallet<T> as FellowshipHandle<AccountIdOf<T>>>::add_to_fellowship(&acc, role, rank, None, false);
                *weight = weight.saturating_add(T::WeightInfo::add_to_fellowship())
            }
        }
    }

    impl<T: Config> OnRuntimeUpgrade for MigrateInitial<T> 
    where 
    <T as frame_system::Config>::AccountId: FromStr,
    <<T as frame_system::Config>::AccountId as FromStr>::Err : Debug

    {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
            log::warn!( target: "pallet-fellowship", "Running pre_upgrade()");
            Ok(Vec::new())
        }

        fn on_runtime_upgrade() -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 1);
            log::warn!("****** STARTING MIGRATION *****");

            let current = <Pallet<T> as GetStorageVersion>::current_storage_version();

            if current == 1 {
                Self::insert_initial_fellows(&mut weight);

                log::warn!("v1 has been successfully applied");
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 1));
            } else {
                log::warn!("Skipping v1, should be removed from Executive");
                weight = weight.saturating_add(T::DbWeight::get().reads(1));
            }

            log::warn!("****** ENDING MIGRATION *****");
            weight
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            log::warn!( target:  "pallet-fellowship", "Running post_upgrade()");

            ensure!(
                Pallet::<T>::current_storage_version() == 1,
                "Storage version should be v1 after the migration"
            );

            Ok(())
        }

    }

    
}
