sp_api::decl_runtime_apis! {
    #[api_version(1)]
    pub trait ProjectsApi<AccountId> where AccountId: codec::Codec {
        fn get_project_account_by_id(project_id: u32) -> AccountId;
    }
}
