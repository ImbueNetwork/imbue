#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_grants.
pub trait WeightInfo {
    fn submit_initial_grant() -> Weight;
}

/// Weights for pallet_proposals using the Substrate node, recommended hardware should be used.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn submit_initial_grant() -> Weight {
        Weight::from_ref_time(49_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn submit_initial_grant() -> Weight {
        Weight::from_ref_time(49_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
}
