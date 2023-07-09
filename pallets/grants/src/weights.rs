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
        Weight::from_ref_time(212_709_000)
            .saturating_add(Weight::from_proof_size(10360))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(7))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_and_convert() -> Weight {
        Weight::from_ref_time(212_709_000)
            .saturating_add(Weight::from_proof_size(10360))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(7))
        }
}
