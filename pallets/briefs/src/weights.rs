#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_grants.
pub trait WeightInfo {
    fn add_to_fellowship() -> Weight;
    fn create_brief() -> Weight;
    fn contribute_to_brief() -> Weight;
    fn commence_work() -> Weight;
}

/// Weights for pallet_proposals using the Substrate node, recommended hardware should be used.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn add_to_fellowship() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn create_brief() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn contribute_to_brief() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn commence_work() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn add_to_fellowship() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn create_brief() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn contribute_to_brief() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn commence_work() -> Weight {
        Weight::from_parts(49_000_000,0)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
}
