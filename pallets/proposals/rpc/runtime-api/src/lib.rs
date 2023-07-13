#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
    pub trait ProposalsApi<AccountId> where AccountId: codec::Codec {
        fn get_project_account_by_id(project_id: u32) -> AccountId;
    }
}
