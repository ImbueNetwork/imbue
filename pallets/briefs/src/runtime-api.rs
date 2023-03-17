use crate::pallet::*;

sp_api::decl_runtime_apis! {
    pub trait GetRemainingBounty<Balance> {
        fn get_remaining_bounty() -> Balance;
    }
}

// TODO IMPL into RUNTIME
//impl sum_storage_rpc_runtime_api::GetRemainingBounty<Balance> for Runtime {
//    fn get_remaining_bounty(brief_id: BriefId) -> Balance {
//        BriefsMod::get_remaining_bounty(brief_id)
//    }
//}