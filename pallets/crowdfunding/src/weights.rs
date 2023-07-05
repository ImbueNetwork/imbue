#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_grants.
pub trait WeightInfo {
    
    fn create_crowdfund() -> Weight;
    fn update_crowdfund() -> Weight;
    fn add_crowdfund_whitelist() -> Weight;
    fn remove_crowdfund_whitelist() -> Weight;
    fn open_contributions() -> Weight;
    fn contribute() -> Weight;
    fn approve_crowdfund_for_milestone_submission() -> Weight;
    
}

/// Weights for pallet_proposals using the Substrate node, recommended hardware should be used.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_crowdfund() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn update_crowdfund() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn add_crowdfund_whitelist() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    fn remove_crowdfund_whitelist() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    fn open_contributions() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn contribute() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn approve_crowdfund_for_milestone_submission() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_crowdfund() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn update_crowdfund() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn add_crowdfund_whitelist() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn remove_crowdfund_whitelist() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn open_contributions() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn contribute() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn approve_crowdfund_for_milestone_submission() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
}
