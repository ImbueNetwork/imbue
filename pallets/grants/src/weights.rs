#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_grants.
pub trait WeightInfo {
    fn create_and_convert() -> Weight;
}

/// Weights for pallet_proposals using the Substrate node, recommended hardware should be used.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_and_convert() -> Weight {
        Weight::from_parts(2_771_742_000, 6128)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(7_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_and_convert() -> Weight {
        Weight::from_parts(2_771_742_000, 6128)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(7_u64))
    }
}
