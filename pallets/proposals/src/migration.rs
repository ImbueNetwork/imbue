use frame_support::{pallet_prelude::OptionQuery, storage_alias, traits::Get, weights::Weight};

use crate::*;
pub use pallet::*;

pub mod v0 {
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
        pub contributions: Vec<ContributionV0<AccountId, Balance>>,
        pub currency_id: common_types::CurrencyId,
        pub required_funds: Balance,
        pub withdrawn_funds: Balance,
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

// Depricated but maintained in case. I dont believe we have a need for this however
pub mod v1 {
    use super::*;

    pub type ProjectV1Of<T> = ProjectV1<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>>;

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
    pub struct ProjectV1<AccountId, Balance, BlockNumber, Timestamp> {
        pub name: Vec<u8>,
        pub logo: Vec<u8>,
        pub description: Vec<u8>,
        pub website: Vec<u8>,
        pub milestones: BTreeMap<MilestoneKey, Milestone>,
        pub contributions: BTreeMap<AccountId, Contribution<Balance, Timestamp>>,
        pub currency_id: common_types::CurrencyId,
        pub required_funds: Balance,
        pub withdrawn_funds: Balance,
        pub raised_funds: Balance,
        pub initiator: AccountId,
        pub create_block_number: BlockNumber,
        pub approved_for_funding: bool,
        pub funding_threshold_met: bool,
        pub cancelled: bool,
    }
    
    #[storage_alias]
    pub type Projects<T: Config> =
        StorageMap<Pallet<T>, Identity, ProjectKey, ProjectV1Of<T>, OptionQuery>;

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

            let _migrated_project: ProjectV1<
                T::AccountId,
                BalanceOf<T>,
                T::BlockNumber,
                TimestampOf<T>,
            > = ProjectV1 {
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
            None
            // DEPRICATED
            //Some(migrated_project)
        });
        weight
    }
}

pub mod v2 {
    use super::*;
    use v1::{ProjectV1, ProjectV1Of};

    #[storage_alias]
    pub type Projects<T: Config> =
        StorageMap<Pallet<T>, Identity, ProjectKey, Project<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>>, OptionQuery>;


    pub fn migrate<T: Config>() -> Weight {
        let mut weight: Weight = Default::default();

        Projects::<T>::translate(|_project_key, project: v1::ProjectV1Of<T>| { 
            weight += T::DbWeight::get().reads_writes(1, 1);
            let migrated_project = Project::<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>> {
                name: project.name,
                logo: project.logo,
                description: project.description,
                website: project.website,
                milestones: project.milestones,
                contributions: project.contributions,
                required_funds: project.required_funds,
                currency_id: project.currency_id,
                withdrawn_funds: project.withdrawn_funds,
                initiator: project.initiator,
                create_block_number: project.create_block_number,
                approved_for_funding: project.approved_for_funding,
                funding_threshold_met: project.funding_threshold_met,
                cancelled: project.cancelled,
                raised_funds: project.raised_funds,
                // Migrate over the new field fee taken set as default.
                fee_taken: Default::default(),
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
    use v1::{ProjectV1, ProjectV1Of};


    #[test]
    fn migrate_v1_to_v2() {
        let contribution_value = 10_000_00u64;

        build_test_externality().execute_with(|| {
            let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
            let bob = get_account_id_from_seed::<sr25519::Public>("Bob");

            let project_key = 10;

            let old_project = ProjectV1Of::<Test> {
                name: b"Project Pre-migrations".to_vec().try_into().unwrap(),
                logo: b"logo".to_vec().try_into().unwrap(),
                description: b"description".to_vec().try_into().unwrap(),
                website: b"https://imbue.network".to_vec().try_into().unwrap(),
                milestones: BTreeMap::new(),
                contributions: BTreeMap::new(),
                currency_id: CurrencyId::KSM,
                required_funds: (100_000_000u32).into(),
                withdrawn_funds: (0u32).into(),
                initiator: alice,
                create_block_number: 100u64,
                approved_for_funding: true,
                funding_threshold_met: true,
                cancelled: false,
                raised_funds: Default::default(),
            };
            dbg!(&old_project);

            v1::Projects::<Test>::insert(project_key, &old_project);
            dbg!(&v2::Projects::<Test>::iter_keys().collect::<Vec<_>>());
            dbg!(&v1::Projects::<Test>::iter_keys().collect::<Vec<_>>());
            dbg!(&Projects::<Test>::iter_keys().collect::<Vec<_>>());

            
            let _ = v2::migrate::<Test>();
            
            let migrated_project = v2::Projects::<Test>::get(&project_key).unwrap();

            assert_eq!(old_project.name, migrated_project.name);

            assert_eq!(
                &old_project.milestones[&0u32],
                migrated_project.milestones.get(&0).unwrap()
            );

            assert_eq!(
                contribution_value.saturating_mul(2),
                migrated_project.raised_funds
            );
            dbg!(&migrated_project);
            assert!(false);
            //assert_eq!(
            //    migrated_project.fee_taken,
            //    Default::default()
            //)
        })
    }
}
