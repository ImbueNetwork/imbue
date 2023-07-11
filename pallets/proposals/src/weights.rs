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
        Weight::from_parts(752_706_000, 266244)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    fn vote_on_milestone() -> Weight {
        Weight::from_parts(973_777_000, 435823)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    fn withdraw() -> Weight {
        Weight::from_parts(2_547_270_000, 273651)
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    fn raise_vote_of_no_confidence() -> Weight {
        Weight::from_parts(703_535_000, 436249)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    fn vote_on_no_confidence_round() -> Weight {
        Weight::from_parts(686_616_000, 435819)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn finalise_no_confidence_round() -> Weight {
        Weight::from_parts(2_454_429_000, 281282)
            .saturating_add(T::DbWeight::get().reads(8_u64))
            .saturating_add(T::DbWeight::get().writes(6_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn submit_milestone() -> Weight {
        Weight::from_parts(752_706_000, 266244)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
    }
    fn vote_on_milestone() -> Weight {
        Weight::from_parts(973_777_000, 435823)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
    fn withdraw() -> Weight {
        Weight::from_parts(2_547_270_000, 273651)
            .saturating_add(RocksDbWeight::get().reads(5_u64))
            .saturating_add(RocksDbWeight::get().writes(5_u64))
    }
    fn raise_vote_of_no_confidence() -> Weight {
        Weight::from_parts(703_535_000, 436249)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
    }
    fn vote_on_no_confidence_round() -> Weight {
        Weight::from_parts(686_616_000, 435819)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn finalise_no_confidence_round() -> Weight {
        Weight::from_parts(2_454_429_000, 281282)
            .saturating_add(RocksDbWeight::get().reads(8_u64))
            .saturating_add(RocksDbWeight::get().writes(6_u64))
    }
}
