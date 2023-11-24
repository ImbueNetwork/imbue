use super::*;
use frame_support::traits::OnRuntimeUpgrade;
use frame_support::{pallet_prelude::*, *};
use frame_system::pallet_prelude::BlockNumberFor;
use hex_literal::hex;
use sp_runtime::AccountId32;
use sp_std::{vec, vec::Vec};

use crate::traits::*;

pub mod v0 {
    use super::*;

    pub struct MigrateInitial<T: crate::Config>(T);
    impl<T: Config> MigrateInitial<T>
    where
        T: frame_system::Config<AccountId = AccountId32>,
    {
        pub fn insert_initial_fellows(
            weight: &mut Weight,
            initial_fellows: Vec<(AccountIdOf<T>, crate::Role, crate::Rank)>,
        ) {
            for (acc, role, rank) in initial_fellows.into_iter() {
                <Pallet<T> as FellowshipHandle<AccountIdOf<T>>>::add_to_fellowship(
                    &acc, role, rank, None, false,
                );
                *weight = weight.saturating_add(T::WeightInfo::add_to_fellowship())
            }
        }
        pub fn get_initial_fellows() -> Vec<(AccountIdOf<T>, crate::Role, crate::Rank)> {
            vec![
                // EARNEST
                //"5Da1Fna8wvgQNmCFPhcRGR9oxmhyPd7MNhPZADq2X6GiKkkr",
                (
                    AccountId32::new(hex![
                        "4294eb45758b4b92b01ceffe209bbcfeb26c973d5c0e21ac6c9cfbb99201b334"
                    ]),
                    Role::Freelancer,
                    1,
                ),
                // FELIX
                //  5DCzKK5EZvY77vxxWXeip7sp17TqB7sk7Fj1hXes7Bo6B5Eq
                (
                    AccountId32::new(hex![
                        "328d9a97c6f7f0fbbc60be2faba4c36cd4e5d3cfcb316393b384ee1a45433034"
                    ]),
                    Role::Freelancer,
                    1,
                ),
                // BEA
                // "5DU2hcQnEmrSXCDUnjiwNX3A1uTf26ACpgs4KUFpsLJqAnjd",
                (
                    AccountId32::new(hex![
                        "3e064fcfd9f02b99dda26226d3d6b2d68032b1c990e7a350cd01747271356f4c"
                    ]),
                    Role::Freelancer,
                    1,
                ),
                //  SAM
                //  "5F28xL42VWThNonDft4TAQ6rw6a82E2jMsQXS5uMyKiA4ccv",
                (
                    AccountId32::new(hex![
                        "82bf733f44a840f0a5c1935a002d4e541d81298fad6d1da8124073485983860e"
                    ]),
                    Role::Freelancer,
                    1,
                ),
                //  SHANKAR
                //  "5E6pjCAGAtpV4nDoTWfMyQ474ku9DNScYeU3PK3e8Jd94Z1n",
                (
                    AccountId32::new(hex![
                        "5a1616831e4508abf2eced2670199ab7a00e9e2bbcfc04655ba7ed138af8787d"
                    ]),
                    Role::Freelancer,
                    1,
                ),
            ]
        }
    }

    impl<T: Config> OnRuntimeUpgrade for MigrateInitial<T>
    where
        T: frame_system::Config<AccountId = AccountId32>,
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
            let onchain = <Pallet<T> as GetStorageVersion>::on_chain_storage_version();

            if current == 1 && onchain == 0 {
                let initial_fellows = Self::get_initial_fellows();
                Self::insert_initial_fellows(&mut weight, initial_fellows);

                current.put::<Pallet<T>>();
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

            let accounts = vec![
                AccountId32::new(hex![
                    "4294eb45758b4b92b01ceffe209bbcfeb26c973d5c0e21ac6c9cfbb99201b334"
                ]),
                AccountId32::new(hex![
                    "328d9a97c6f7f0fbbc60be2faba4c36cd4e5d3cfcb316393b384ee1a45433034"
                ]),
                AccountId32::new(hex![
                    "3e064fcfd9f02b99dda26226d3d6b2d68032b1c990e7a350cd01747271356f4c"
                ]),
                AccountId32::new(hex![
                    "82bf733f44a840f0a5c1935a002d4e541d81298fad6d1da8124073485983860e"
                ]),
                AccountId32::new(hex![
                    "5a1616831e4508abf2eced2670199ab7a00e9e2bbcfc04655ba7ed138af8787d"
                ]),
            ];

            accounts.iter().for_each(|acc| {
                let role = Roles::<T>::get(acc).unwrap();
                assert!(
                    role == (Role::Freelancer, 1),
                    "Roles have not been inserted correctly."
                );
            });

            ensure!(
                Pallet::<T>::current_storage_version() == 1,
                "Storage version should be v1 after the migration"
            );

            Ok(())
        }
    }
}
