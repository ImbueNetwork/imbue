

use frame_support::pallet_prelude::Weight;

pub struct WeightInfo;
impl crate::traits::WeightInfoT for WeightInfo {
    fn add_to_fellowship() -> Weight {
        <Weight as Default>::default()
    }
    fn force_add_fellowship() -> Weight {
        <Weight as Default>::default()
    }
    fn leave_fellowship() -> Weight {
        <Weight as Default>::default()
    }
    fn force_remove_and_slash_fellowship() -> Weight {
        <Weight as Default>::default()
    }
    fn add_candidate_to_shortlist() -> Weight {
        <Weight as Default>::default()
    }
    fn remove_candidate_from_shortlist() -> Weight {
        <Weight as Default>::default()
    }
    fn pay_deposit_to_remove_pending_status() -> Weight {
        <Weight as Default>::default()
    }
}
