
//! Autogenerated weights for `pallet_proposals`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-08-03, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `user`, CPU: `12th Gen Intel(R) Core(TM) i9-12900H`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("local"), DB CACHE: 1024

// Executed Command:
// ./target/release/imbue
// benchmark
// pallet
// --chain
// local
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet-proposals
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

/// Weight functions for `pallet_proposals`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_proposals::WeightInfo for WeightInfo<T> {
	/// Storage: ImbueProposals Projects (r:1 w:0)
	/// Proof: ImbueProposals Projects (max_values: None, max_size: Some(260823), added: 263298, mode: MaxEncodedLen)
	/// Storage: ImbueProposals RoundsExpiring (r:1 w:1)
	/// Proof: ImbueProposals RoundsExpiring (max_values: None, max_size: Some(471), added: 2946, mode: MaxEncodedLen)
	/// Storage: ImbueProposals Rounds (r:0 w:1)
	/// Proof: ImbueProposals Rounds (max_values: None, max_size: Some(41), added: 2516, mode: MaxEncodedLen)
	/// Storage: ImbueProposals MilestoneVotes (r:0 w:1)
	/// Proof: ImbueProposals MilestoneVotes (max_values: None, max_size: Some(41), added: 2516, mode: MaxEncodedLen)
	/// Storage: ImbueProposals UserHasVoted (r:0 w:1)
	/// Proof: ImbueProposals UserHasVoted (max_values: None, max_size: Some(165018), added: 167493, mode: MaxEncodedLen)
	fn submit_milestone() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `961`
		//  Estimated: `264288`
		// Minimum execution time: 22_603_000 picoseconds.
		Weight::from_parts(23_419_000, 0)
			.saturating_add(Weight::from_parts(0, 264288))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: ImbueProposals Projects (r:1 w:1)
	/// Proof: ImbueProposals Projects (max_values: None, max_size: Some(260823), added: 263298, mode: MaxEncodedLen)
	/// Storage: ImbueProposals Rounds (r:1 w:1)
	/// Proof: ImbueProposals Rounds (max_values: None, max_size: Some(41), added: 2516, mode: MaxEncodedLen)
	/// Storage: ImbueProposals UserHasVoted (r:1 w:1)
	/// Proof: ImbueProposals UserHasVoted (max_values: None, max_size: Some(165018), added: 167493, mode: MaxEncodedLen)
	/// Storage: ImbueProposals MilestoneVotes (r:1 w:1)
	/// Proof: ImbueProposals MilestoneVotes (max_values: None, max_size: Some(41), added: 2516, mode: MaxEncodedLen)
	fn vote_on_milestone() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1100`
		//  Estimated: `264288`
		// Minimum execution time: 31_914_000 picoseconds.
		Weight::from_parts(32_273_000, 0)
			.saturating_add(Weight::from_parts(0, 264288))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: ImbueProposals Projects (r:1 w:1)
	/// Proof: ImbueProposals Projects (max_values: None, max_size: Some(260823), added: 263298, mode: MaxEncodedLen)
	/// Storage: System Account (r:3 w:3)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Deposits CurrentDeposits (r:1 w:1)
	/// Proof: Deposits CurrentDeposits (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: ImbueProposals CompletedProjects (r:1 w:1)
	/// Proof: ImbueProposals CompletedProjects (max_values: None, max_size: Some(262184), added: 264659, mode: MaxEncodedLen)
	fn withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1567`
		//  Estimated: `265649`
		// Minimum execution time: 119_458_000 picoseconds.
		Weight::from_parts(121_291_000, 0)
			.saturating_add(Weight::from_parts(0, 265649))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: ImbueProposals Projects (r:1 w:0)
	/// Proof: ImbueProposals Projects (max_values: None, max_size: Some(260823), added: 263298, mode: MaxEncodedLen)
	/// Storage: ImbueProposals NoConfidenceVotes (r:1 w:1)
	/// Proof: ImbueProposals NoConfidenceVotes (max_values: None, max_size: Some(37), added: 2512, mode: MaxEncodedLen)
	/// Storage: ImbueProposals RoundsExpiring (r:1 w:1)
	/// Proof: ImbueProposals RoundsExpiring (max_values: None, max_size: Some(471), added: 2946, mode: MaxEncodedLen)
	/// Storage: ImbueProposals UserHasVoted (r:1 w:1)
	/// Proof: ImbueProposals UserHasVoted (max_values: None, max_size: Some(165018), added: 167493, mode: MaxEncodedLen)
	/// Storage: ImbueProposals Rounds (r:0 w:1)
	/// Proof: ImbueProposals Rounds (max_values: None, max_size: Some(41), added: 2516, mode: MaxEncodedLen)
	fn raise_vote_of_no_confidence() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `961`
		//  Estimated: `264288`
		// Minimum execution time: 21_212_000 picoseconds.
		Weight::from_parts(21_625_000, 0)
			.saturating_add(Weight::from_parts(0, 264288))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: ImbueProposals Rounds (r:1 w:0)
	/// Proof: ImbueProposals Rounds (max_values: None, max_size: Some(41), added: 2516, mode: MaxEncodedLen)
	/// Storage: ImbueProposals Projects (r:1 w:0)
	/// Proof: ImbueProposals Projects (max_values: None, max_size: Some(260823), added: 263298, mode: MaxEncodedLen)
	/// Storage: ImbueProposals NoConfidenceVotes (r:1 w:1)
	/// Proof: ImbueProposals NoConfidenceVotes (max_values: None, max_size: Some(37), added: 2512, mode: MaxEncodedLen)
	/// Storage: ImbueProposals UserHasVoted (r:1 w:1)
	/// Proof: ImbueProposals UserHasVoted (max_values: None, max_size: Some(165018), added: 167493, mode: MaxEncodedLen)
	fn vote_on_no_confidence_round() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1223`
		//  Estimated: `264288`
		// Minimum execution time: 21_730_000 picoseconds.
		Weight::from_parts(22_659_000, 0)
			.saturating_add(Weight::from_parts(0, 264288))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ImbueProposals RoundsExpiring (r:1 w:1)
	/// Proof: ImbueProposals RoundsExpiring (max_values: None, max_size: Some(471), added: 2946, mode: MaxEncodedLen)
	/// Storage: ImbueProposals Rounds (r:0 w:1)
	/// Proof: ImbueProposals Rounds (max_values: None, max_size: Some(41), added: 2516, mode: MaxEncodedLen)
	/// Storage: ImbueProposals MilestoneVotes (r:0 w:1)
	/// Proof: ImbueProposals MilestoneVotes (max_values: None, max_size: Some(41), added: 2516, mode: MaxEncodedLen)
	/// Storage: ImbueProposals UserHasVoted (r:0 w:1)
	/// Proof: ImbueProposals UserHasVoted (max_values: None, max_size: Some(165018), added: 167493, mode: MaxEncodedLen)
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `97`
		//  Estimated: `3936`
		// Minimum execution time: 6_577_000 picoseconds.
		Weight::from_parts(6_907_000, 0)
			.saturating_add(Weight::from_parts(0, 3936))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}
