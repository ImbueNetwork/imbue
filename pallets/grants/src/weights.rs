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
    fn edit_grant() -> Weight;
    fn cancel_grant() -> Weight;
    fn convert_to_project() -> Weight;
}

/// Weights for pallet_proposals using the Substrate node, recommended hardware should be used.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn submit_initial_grant() -> Weight {
        Weight::from_ref_time(81_668_000)
			.saturating_add(Weight::from_proof_size(7816))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
    }
    fn edit_grant() -> Weight {
        Weight::from_ref_time(43_368_000)
			.saturating_add(Weight::from_proof_size(4211))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
    }
    fn cancel_grant() -> Weight {
        Weight::from_ref_time(38_783_000)
			.saturating_add(Weight::from_proof_size(4211))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
    }
    fn convert_to_project() -> Weight {
        Weight::from_ref_time(212_709_000)
			.saturating_add(Weight::from_proof_size(10360))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn submit_initial_grant() -> Weight {
        Weight::from_ref_time(81_668_000)
			.saturating_add(Weight::from_proof_size(7816))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(6))
    }
    fn edit_grant() -> Weight {
        Weight::from_ref_time(43_368_000)
			.saturating_add(Weight::from_proof_size(4211))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
    }
    fn cancel_grant() -> Weight {
        Weight::from_ref_time(38_783_000)
			.saturating_add(Weight::from_proof_size(4211))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
    }
    fn convert_to_project() -> Weight {
        Weight::from_ref_time(212_709_000)
			.saturating_add(Weight::from_proof_size(10360))
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(7))
    }
}
