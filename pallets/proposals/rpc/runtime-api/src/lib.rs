#![cfg_attr(not(feature = "std"), no_std)]
use crate::pallet::{MilestoneKey, };

sp_api::decl_runtime_apis! {
    pub trait ProposalsApi<AccountId, Balance> where AccountId: codec::Codec {
        fn get_project_account_by_id(project_id: u32) -> AccountId;
        fn get_project_individuals_votes(project_id: u32) -> BTreeMap<MilestoneKey, BTreeMap<AccountId, (bool, Balance)>>;
        fn get_project_total_votes(project_id: u32) -> BTreeMap<MilestoneKey, Vote<Balance>>;
    }
}
