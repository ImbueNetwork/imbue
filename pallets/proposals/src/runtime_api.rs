sp_api::decl_runtime_apis! {
    pub trait ProjectsApi<AccountId> where AccountId: codec::Codec {
        fn get_project_account_by_id(projet_id: u32) -> AccountId;
    }
}
