#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_proposals.
pub trait WeightInfo {
    fn create_project() -> Weight;
    fn add_project_whitelist() -> Weight; 
    fn remove_project_whitelist() -> Weight; 
    fn schedule_round() -> Weight;
    fn cancel_round() -> Weight;
    fn contribute() -> Weight;
    fn approve() -> Weight;
    fn submit_milestone() -> Weight;
    fn vote_on_milestone() -> Weight;
    fn finalise_milestone_voting() -> Weight;
    fn withdraw() -> Weight;
    fn raise_vote_of_no_confidence() -> Weight;
    fn vote_on_no_confidence_round() -> Weight;
    fn finalise_no_confidence_round() -> Weight;
    fn set_storage_variable() -> Weight;
    fn refund() -> Weight;
    fn fund() -> Weight;
    fn refund_item_in_queue() -> Weight;
    fn split_off_refunds() -> Weight;
}


/// Weights for pallet_proposals using the Substrate node, recommended hardware should be used.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_project() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn add_project_whitelist() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn remove_project_whitelist() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn schedule_round() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn cancel_round() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn contribute() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn approve() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn submit_milestone() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn vote_on_milestone() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn finalise_milestone_voting() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn withdraw() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn raise_vote_of_no_confidence() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn vote_on_no_confidence_round() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn finalise_no_confidence_round() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn set_storage_variable() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn refund() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn fund() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn refund_item_in_queue() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn split_off_refunds() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_project() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn add_project_whitelist() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn remove_project_whitelist() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn schedule_round() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn cancel_round() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn contribute() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn approve() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn submit_milestone() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn vote_on_milestone() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn finalise_milestone_voting() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn withdraw() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn raise_vote_of_no_confidence() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn vote_on_no_confidence_round() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn finalise_no_confidence_round() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn set_storage_variable() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn refund() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn fund() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn split_off_refunds() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn refund_item_in_queue() -> Weight {
        Weight::from_ref_time(49_000_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
}
