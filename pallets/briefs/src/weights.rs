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
    fn cancel_brief() -> Weight;
}

/// Weights for pallet_proposals using the Substrate node, recommended hardware should be used.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn add_to_fellowship() -> Weight {
        Weight::from_parts(246_832_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn create_brief() -> Weight {
        Weight::from_parts(1_570_112_000, 17171)
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(6_u64))
    }
    fn contribute_to_brief() -> Weight {
        Weight::from_parts(884_477_000, 16169)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn commence_work() -> Weight {
        Weight::from_parts(3_477_167_000, 25420)
            .saturating_add(T::DbWeight::get().reads(9_u64))
            .saturating_add(T::DbWeight::get().writes(11_u64))
    }
    fn cancel_brief() -> Weight {
        Weight::from_parts(1_651_543_000, 19212)
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn add_to_fellowship() -> Weight {
        Weight::from_parts(246_832_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn create_brief() -> Weight {
        Weight::from_parts(1_570_112_000, 17171)
            .saturating_add(RocksDbWeight::get().reads(5_u64))
            .saturating_add(RocksDbWeight::get().writes(6_u64))
    }
    fn contribute_to_brief() -> Weight {
        Weight::from_parts(884_477_000, 16169)
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn commence_work() -> Weight {
        Weight::from_parts(3_477_167_000, 25420)
            .saturating_add(RocksDbWeight::get().reads(9_u64))
            .saturating_add(RocksDbWeight::get().writes(11_u64))
    }
    fn cancel_brief() -> Weight {
        Weight::from_parts(1_651_543_000, 19212)
            .saturating_add(RocksDbWeight::get().reads(5_u64))
            .saturating_add(RocksDbWeight::get().writes(5_u64))
    }
}
