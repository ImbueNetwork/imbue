use frame_support::{pallet_prelude::OptionQuery, storage_alias, traits::Get, weights::Weight};

use crate::*;
pub use pallet::*;

mod v0 {
    use super::*;
    pub type ProjectV0Of<T> = ProjectV0<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>;

    #[derive(Encode, Clone, Decode)]
    pub struct ContributionV0<AccountId, Balance> {
        pub account_id: AccountId,
        pub value: Balance,
    }

    #[derive(Encode, Clone, Decode)]
    pub struct ProjectV0<AccountId, Balance, BlockNumber> {
        pub name: Vec<u8>,
        pub logo: Vec<u8>,
        pub description: Vec<u8>,
        pub website: Vec<u8>,
        pub milestones: Vec<Milestone>,
        /// A collection of the accounts which have contributed and their contributions.
        pub contributions: Vec<ContributionV0<AccountId, Balance>>,
        pub currency_id: common_types::CurrencyId,
        pub required_funds: Balance,
        pub withdrawn_funds: Balance,
        /// The account that will receive the funds if the campaign is successful
        pub initiator: AccountId,
        pub create_block_number: BlockNumber,
        pub approved_for_funding: bool,
        pub funding_threshold_met: bool,
        pub cancelled: bool,
    }

    #[storage_alias]
    pub type Projects<T: Config> =
        StorageMap<Pallet<T>, Identity, ProjectKey, ProjectV0Of<T>, OptionQuery>;
}

pub mod v1 {
    use super::*;

    pub fn migrate<T: Config>() -> Weight {
        let mut weight = T::DbWeight::get().reads_writes(1, 1);

        Projects::<T>::translate(|_project_key, project: v0::ProjectV0Of<T>| {
            weight += T::DbWeight::get().reads_writes(1, 1);

            let mut migrated_contributions: BTreeMap<
                AccountIdOf<T>,
                Contribution<BalanceOf<T>, TimestampOf<T>>,
            > = BTreeMap::new();
            let mut migrated_milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();

            let mut raised_funds: BalanceOf<T> = (0u32).into();

            let _ = project
                .contributions
                .into_iter()
                .map(|contribution| {
                    let migrated_contribution = Contribution {
                        value: contribution.value,
                        timestamp: TimestampOf::<T>::default(),
                    };
                    migrated_contributions.insert(contribution.account_id, migrated_contribution);
                    raised_funds += contribution.value
                })
                .collect::<Vec<_>>();

            let _ = project
                .milestones
                .into_iter()
                .map(|milestone| {
                    migrated_milestones.insert(milestone.milestone_key, milestone.clone())
                })
                .collect::<Vec<_>>();

            let migrated_project: Project<
                T::AccountId,
                BalanceOf<T>,
                T::BlockNumber,
                TimestampOf<T>,
            > = Project {
                name: project.name,
                logo: project.logo,
                description: project.description,
                website: project.website,
                milestones: migrated_milestones,
                contributions: migrated_contributions,
                required_funds: project.required_funds,
                currency_id: project.currency_id,
                withdrawn_funds: project.withdrawn_funds,
                initiator: project.initiator,
                create_block_number: project.create_block_number,
                approved_for_funding: project.approved_for_funding,
                funding_threshold_met: project.funding_threshold_met,
                cancelled: project.cancelled,
                raised_funds: raised_funds,
            };
            Some(migrated_project)
        });
        weight
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mock::*;
    use sp_core::sr25519;
    use sp_std::vec::Vec;
    use v0::{ContributionV0, ProjectV0};

    #[test]
    fn migrate_v0_to_v1() {
        let contribution_value = 10_000_00u64;

        build_test_externality().execute_with(|| {
            let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
            let bob = get_account_id_from_seed::<sr25519::Public>("Bob");

            let project_key = 1;

            let old_milestones = vec![
                Milestone {
                    project_key,
                    milestone_key: 0,
                    name: Vec::new(),
                    percentage_to_unlock: 40,
                    is_approved: true,
                },
                Milestone {
                    project_key,
                    milestone_key: 1,
                    name: Vec::new(),
                    percentage_to_unlock: 60,
                    is_approved: true,
                },
            ];

            let old_contributions = vec![
                ContributionV0 {
                    account_id: alice,
                    value: contribution_value,
                },
                ContributionV0 {
                    account_id: bob,
                    value: contribution_value,
                },
            ];

            let old_project = ProjectV0 {
                name: b"Project Pre-migrations".to_vec(),
                logo: b"logo".to_vec(),
                description: b"description".to_vec(),
                website: b"https://imbue.network".to_vec(),
                milestones: old_milestones,
                contributions: old_contributions,
                currency_id: CurrencyId::KSM,
                required_funds: (100_000_000u32).into(),
                withdrawn_funds: (0u32).into(),
                initiator: alice,
                create_block_number: 100u64,
                approved_for_funding: true,
                funding_threshold_met: true,
                cancelled: false,
            };

            v0::Projects::<Test>::insert(project_key, &old_project);
            let _ = v1::migrate::<Test>();
            let migrated_project = Projects::<Test>::get(&project_key).unwrap();

            assert_eq!(old_project.name, migrated_project.name);

            assert_eq!(
                &old_project.milestones[0],
                migrated_project.milestones.get(&0).unwrap()
            );

            assert_eq!(
                &old_project.contributions[0].value,
                &migrated_project.contributions.get(&alice).unwrap().value
            );

            assert_eq!(
                contribution_value.saturating_mul(2),
                migrated_project.raised_funds
            );
        })
    }
}
