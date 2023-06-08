#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_proposals.
pub trait WeightInfo {
    fn submit_milestone() -> Weight;
    fn vote_on_milestone() -> Weight;
    fn withdraw() -> Weight;
    fn raise_vote_of_no_confidence() -> Weight;
    fn vote_on_no_confidence_round() -> Weight;
    fn finalise_no_confidence_round() -> Weight;
}

/// Weights for pallet_proposals using the Substrate node, recommended hardware should be used.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn submit_milestone() -> Weight {
        Weight::from_ref_time(51_777_000)
			.saturating_add(Weight::from_proof_size(266244))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
    }
    fn vote_on_milestone() -> Weight {
        Weight::from_ref_time(65_447_000)
			.saturating_add(Weight::from_proof_size(435823))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
    }
    fn withdraw() -> Weight {
        Weight::from_ref_time(136_071_000)
			.saturating_add(Weight::from_proof_size(271107))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
    }
    fn raise_vote_of_no_confidence() -> Weight {
        Weight::from_ref_time(46_597_000)
			.saturating_add(Weight::from_proof_size(436249))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
    }
    fn vote_on_no_confidence_round() -> Weight {
        Weight::from_ref_time(50_107_000)
			.saturating_add(Weight::from_proof_size(435819))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
    }
    fn finalise_no_confidence_round() -> Weight {
        Weight::from_ref_time(168_434_000)
			.saturating_add(Weight::from_proof_size(281282))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(6))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn submit_milestone() -> Weight {
        Weight::from_ref_time(51_777_000)
			.saturating_add(Weight::from_proof_size(266244))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(4))
    }
    fn vote_on_milestone() -> Weight {
        Weight::from_ref_time(65_447_000)
			.saturating_add(Weight::from_proof_size(435823))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(3))
    }
    fn withdraw() -> Weight {
        Weight::from_ref_time(136_071_000)
			.saturating_add(Weight::from_proof_size(271107))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(4))
    }
    fn raise_vote_of_no_confidence() -> Weight {
        Weight::from_ref_time(46_597_000)
			.saturating_add(Weight::from_proof_size(436249))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(4))
    }
    fn vote_on_no_confidence_round() -> Weight {
        Weight::from_ref_time(50_107_000)
			.saturating_add(Weight::from_proof_size(435819))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(2))
    }
    fn finalise_no_confidence_round() -> Weight {
        Weight::from_ref_time(168_434_000)
			.saturating_add(Weight::from_proof_size(281282))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(6))
    }
}
