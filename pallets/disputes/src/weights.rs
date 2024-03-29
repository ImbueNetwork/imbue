
//! Autogenerated weights for `pallet_disputes`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-24, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `user`, CPU: `12th Gen Intel(R) Core(TM) i9-12900H`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("local")`, DB CACHE: 1024

// Executed Command:
// ./target/debug/imbue
// benchmark
// pallet
// --chain
// local
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet-disputes
// --extrinsic
// *
// --output
// weights.rs
// --steps
// 50
// --repeat
// 20

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_disputes`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> crate::WeightInfoT for WeightInfo<T> {
	/// Storage: `ImbueDisputes::Disputes` (r:1 w:1)
	/// Proof: `ImbueDisputes::Disputes` (`max_values`: None, `max_size`: Some(6602), added: 9077, mode: `MaxEncodedLen`)
	/// Storage: `ImbueDisputes::DisputesFinaliseOn` (r:1 w:1)
	/// Proof: `ImbueDisputes::DisputesFinaliseOn` (`max_values`: None, `max_size`: Some(221), added: 2696, mode: `MaxEncodedLen`)
	fn raise_dispute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `10067`
		// Minimum execution time: 152_261_000 picoseconds.
		Weight::from_parts(153_191_000, 0)
			.saturating_add(Weight::from_parts(0, 10067))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `ImbueDisputes::Disputes` (r:1 w:1)
	/// Proof: `ImbueDisputes::Disputes` (`max_values`: None, `max_size`: Some(6602), added: 9077, mode: `MaxEncodedLen`)
	/// Storage: `ImbueDisputes::DisputesFinaliseOn` (r:2 w:2)
	/// Proof: `ImbueDisputes::DisputesFinaliseOn` (`max_values`: None, `max_size`: Some(221), added: 2696, mode: `MaxEncodedLen`)
	fn extend_dispute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `250`
		//  Estimated: `10067`
		// Minimum execution time: 244_000_000 picoseconds.
		Weight::from_parts(245_568_000, 0)
			.saturating_add(Weight::from_parts(0, 10067))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `ImbueDisputes::Disputes` (r:1 w:1)
	/// Proof: `ImbueDisputes::Disputes` (`max_values`: None, `max_size`: Some(6602), added: 9077, mode: `MaxEncodedLen`)
	/// Storage: `ImbueDisputes::DisputesFinaliseOn` (r:1 w:1)
	/// Proof: `ImbueDisputes::DisputesFinaliseOn` (`max_values`: None, `max_size`: Some(221), added: 2696, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::Projects` (r:1 w:1)
	/// Proof: `ImbueProposals::Projects` (`max_values`: None, `max_size`: Some(36350), added: 38825, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::ProjectsInDispute` (r:0 w:1)
	/// Proof: `ImbueProposals::ProjectsInDispute` (`max_values`: None, `max_size`: Some(61), added: 2536, mode: `MaxEncodedLen`)
	fn vote_on_dispute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `292`
		//  Estimated: `39815`
		// Minimum execution time: 337_396_000 picoseconds.
		Weight::from_parts(344_127_000, 0)
			.saturating_add(Weight::from_parts(0, 39815))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `ImbueDisputes::Disputes` (r:1 w:1)
	/// Proof: `ImbueDisputes::Disputes` (`max_values`: None, `max_size`: Some(6602), added: 9077, mode: `MaxEncodedLen`)
	/// Storage: `ImbueDisputes::DisputesFinaliseOn` (r:1 w:1)
	/// Proof: `ImbueDisputes::DisputesFinaliseOn` (`max_values`: None, `max_size`: Some(221), added: 2696, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::Projects` (r:1 w:1)
	/// Proof: `ImbueProposals::Projects` (`max_values`: None, `max_size`: Some(36350), added: 38825, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::ProjectsInDispute` (r:0 w:1)
	/// Proof: `ImbueProposals::ProjectsInDispute` (`max_values`: None, `max_size`: Some(61), added: 2536, mode: `MaxEncodedLen`)
	fn force_fail_dispute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `292`
		//  Estimated: `39815`
		// Minimum execution time: 244_177_000 picoseconds.
		Weight::from_parts(250_254_000, 0)
			.saturating_add(Weight::from_parts(0, 39815))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `ImbueDisputes::Disputes` (r:1 w:1)
	/// Proof: `ImbueDisputes::Disputes` (`max_values`: None, `max_size`: Some(6602), added: 9077, mode: `MaxEncodedLen`)
	/// Storage: `ImbueDisputes::DisputesFinaliseOn` (r:1 w:1)
	/// Proof: `ImbueDisputes::DisputesFinaliseOn` (`max_values`: None, `max_size`: Some(221), added: 2696, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::Projects` (r:1 w:1)
	/// Proof: `ImbueProposals::Projects` (`max_values`: None, `max_size`: Some(36350), added: 38825, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::ProjectsInDispute` (r:0 w:1)
	/// Proof: `ImbueProposals::ProjectsInDispute` (`max_values`: None, `max_size`: Some(61), added: 2536, mode: `MaxEncodedLen`)
	fn force_succeed_dispute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `292`
		//  Estimated: `39815`
		// Minimum execution time: 243_762_000 picoseconds.
		Weight::from_parts(250_041_000, 0)
			.saturating_add(Weight::from_parts(0, 39815))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn calculate_winner() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 12_448_000 picoseconds.
		Weight::from_parts(13_334_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
}
