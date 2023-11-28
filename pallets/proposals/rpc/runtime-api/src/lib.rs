#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait ProposalsApi<AccountId, Balance>
    where AccountId: codec::Codec + Ord,
    {
        fn get_project_account_by_id(project_id: u32) -> AccountId;
        fn get_all_project_data(project_id: u32) -> (Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>);
    }
}
