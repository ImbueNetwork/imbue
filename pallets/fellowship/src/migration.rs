use sp_std::{vec, vec::Vec};
use frame_support::traits::OnRuntimeUpgrade;
use frame_support::*;

mod v0 {
    use super::*;

    struct MigrateInitial<T: crate::Config>(T);
    impl<T: Config> MigrateInitial<T> 
    where <T as frame_system::Config>::AccountId : From<[u8; 32]>
    {
        fn get_initial_fellows(initial_fellows: Vec<([u8; 32], crate::Role, crate::Rank)>) -> Vec<(<T as frame_system::Config>::AccountId>, crate::Role, crate::Rank) {
            initial_fellows.iter().map(|(bytes, _, _)|{
                (bytes.into(), _, _)
            }).collect()
        }
    }

    impl<T: Config> OnRuntimeUpgrade for MigrateInitial<T> {
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
                let initial_fellows: Vec<<T as frame_system::Config>::AccountId> = vec![
                    (b"5Da1Fna8wvgQNmCFPhcRGR9oxmhyPd7MNhPZADq2X6GiKkkr", Role::Freelancer, 10),
                    (b"5DCzKK5EZvY77vxxWXeip7sp17TqB7sk7Fj1hXes7Bo6B5Eq", Role::Freelancer, 10),
                ];
                let accounts = Self::get_initial_fellows(initial_fellows);
                for (acc, role, rank) in accounts.iter() {
                    <Pallet<T> as FellowshipHandle>::add_to_fellowship(acc, role, rank, None, false);
                }

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
