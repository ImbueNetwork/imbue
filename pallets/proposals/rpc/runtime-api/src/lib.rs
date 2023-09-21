#![cfg_attr(not(feature = "std"), no_std)]
use pallet_proposals::{MilestoneKey};
use frame_support::{BoundedBTreeMap, pallet_prelude::*};
use sp_api::Decode;
use sp_std::collections::btree_map::BTreeMap;

#[cfg(feature = "std")]
use sp_runtime::serde::{Serialize, Deserialize};
#[cfg_attr(feature = "std", derive(Serialize, Deserialize)) ]
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo, MaxEncodedLen)]
pub struct IndividualVotes<AccountId: Ord, Balance> {
    inner: BTreeMap<u32, BTreeMap<AccountId, (bool, Balance)>>
}

sp_api::decl_runtime_apis! {
    pub trait ProposalsApi<AccountId, Balance> 
    where AccountId: codec::Codec + Ord,
    IndividualVotes<AccountId, Balance>: sp_api::Decode + sp_api::Encode
    {
        fn get_project_account_by_id(project_id: u32) -> AccountId;
        fn get_project_individuals_votes(project_id: u32) -> IndividualVotes<AccountId, Balance>;
    }
}
