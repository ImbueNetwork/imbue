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
        Weight::from_ref_time(17_035_000)
            .saturating_add(Weight::from_proof_size(0))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn create_brief() -> Weight {
        Weight::from_ref_time(107_068_000)
            .saturating_add(Weight::from_proof_size(17171))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(6))
    }
    fn contribute_to_brief() -> Weight {
        Weight::from_ref_time(61_416_000)
            .saturating_add(Weight::from_proof_size(16169))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn commence_work() -> Weight {
        Weight::from_ref_time(206_002_000)
            .saturating_add(Weight::from_proof_size(25420))
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().writes(11))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn add_to_fellowship() -> Weight {
        Weight::from_ref_time(17_035_000)
            .saturating_add(Weight::from_proof_size(0))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn create_brief() -> Weight {
        Weight::from_ref_time(107_068_000)
            .saturating_add(Weight::from_proof_size(17171))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(6))
    }
    fn contribute_to_brief() -> Weight {
        Weight::from_ref_time(61_416_000)
            .saturating_add(Weight::from_proof_size(16169))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn commence_work() -> Weight {
        Weight::from_ref_time(206_002_000)
            .saturating_add(Weight::from_proof_size(25420))
            .saturating_add(RocksDbWeight::get().reads(9))
            .saturating_add(RocksDbWeight::get().writes(11))
    }
}
