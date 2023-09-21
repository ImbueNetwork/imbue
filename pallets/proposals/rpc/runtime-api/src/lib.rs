#![cfg_attr(not(feature = "std"), no_std)]
use pallet_proposals::MilestoneKey;
use frame_support::{BoundedBTreeMap, pallet_prelude::Get};
use sp_api::Decode;



sp_api::decl_runtime_apis! {
    pub trait ProposalsApi<AccountId, Balance, MaxMilestonesPerProject: Get<u32>, MaximumContributorsPerProject: Get<u32>> 
    where AccountId: codec::Codec,
    BoundedBTreeMap<u32, BoundedBTreeMap<AccountId, (bool, Balance), MaximumContributorsPerProject>, MaxMilestonesPerProject>: Decode

    {
        fn get_project_account_by_id(project_id: u32) -> AccountId;
        fn get_project_individuals_votes(project_id: u32) -> BoundedBTreeMap<MilestoneKey, BoundedBTreeMap<AccountId, (bool, Balance), MaximumContributorsPerProject>, MaxMilestonesPerProject>;
    }
}
