
//! Autogenerated weights for `pallet_proposals`
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
impl<T: frame_system::Config> crate::WeightInfoT for WeightInfo<T> {
	/// Storage: `ImbueProposals::Projects` (r:1 w:0)
	/// Proof: `ImbueProposals::Projects` (`max_values`: None, `max_size`: Some(36350), added: 38825, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::RoundsExpiring` (r:1 w:1)
	/// Proof: `ImbueProposals::RoundsExpiring` (`max_values`: None, `max_size`: Some(111), added: 2586, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::IndividualVoteStore` (r:1 w:1)
	/// Proof: `ImbueProposals::IndividualVoteStore` (`max_values`: None, `max_size`: Some(16571), added: 19046, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::MilestoneVotes` (r:1 w:1)
	/// Proof: `ImbueProposals::MilestoneVotes` (`max_values`: None, `max_size`: Some(375), added: 2850, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::Rounds` (r:0 w:1)
	/// Proof: `ImbueProposals::Rounds` (`max_values`: None, `max_size`: Some(45), added: 2520, mode: `MaxEncodedLen`)
	fn submit_milestone() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `568`
		//  Estimated: `39815`
		// Minimum execution time: 345_914_000 picoseconds.
		Weight::from_parts(354_103_000, 0)
			.saturating_add(Weight::from_parts(0, 39815))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `ImbueProposals::Projects` (r:1 w:1)
	/// Proof: `ImbueProposals::Projects` (`max_values`: None, `max_size`: Some(36350), added: 38825, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::Rounds` (r:1 w:1)
	/// Proof: `ImbueProposals::Rounds` (`max_values`: None, `max_size`: Some(45), added: 2520, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::IndividualVoteStore` (r:1 w:1)
	/// Proof: `ImbueProposals::IndividualVoteStore` (`max_values`: None, `max_size`: Some(16571), added: 19046, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::MilestoneVotes` (r:1 w:1)
	/// Proof: `ImbueProposals::MilestoneVotes` (`max_values`: None, `max_size`: Some(375), added: 2850, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::RoundsExpiring` (r:0 w:1)
	/// Proof: `ImbueProposals::RoundsExpiring` (`max_values`: None, `max_size`: Some(111), added: 2586, mode: `MaxEncodedLen`)
	fn vote_on_milestone() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `711`
		//  Estimated: `39815`
		// Minimum execution time: 460_847_000 picoseconds.
		Weight::from_parts(472_559_000, 0)
			.saturating_add(Weight::from_parts(0, 39815))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `ImbueProposals::Projects` (r:1 w:1)
	/// Proof: `ImbueProposals::Projects` (`max_values`: None, `max_size`: Some(36350), added: 38825, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:3 w:3)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Deposits::CurrentDeposits` (r:1 w:1)
	/// Proof: `Deposits::CurrentDeposits` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::CompletedProjects` (r:1 w:1)
	/// Proof: `ImbueProposals::CompletedProjects` (`max_values`: None, `max_size`: Some(262184), added: 264659, mode: `MaxEncodedLen`)
	fn withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1120`
		//  Estimated: `265649`
		// Minimum execution time: 1_638_213_000 picoseconds.
		Weight::from_parts(1_677_055_000, 0)
			.saturating_add(Weight::from_parts(0, 265649))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: `ImbueProposals::RoundsExpiring` (r:1 w:1)
	/// Proof: `ImbueProposals::RoundsExpiring` (`max_values`: None, `max_size`: Some(111), added: 2586, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::MilestoneVotes` (r:1 w:1)
	/// Proof: `ImbueProposals::MilestoneVotes` (`max_values`: None, `max_size`: Some(375), added: 2850, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::IndividualVoteStore` (r:1 w:1)
	/// Proof: `ImbueProposals::IndividualVoteStore` (`max_values`: None, `max_size`: Some(16571), added: 19046, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::Rounds` (r:0 w:1)
	/// Proof: `ImbueProposals::Rounds` (`max_values`: None, `max_size`: Some(45), added: 2520, mode: `MaxEncodedLen`)
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `97`
		//  Estimated: `20036`
		// Minimum execution time: 147_964_000 picoseconds.
		Weight::from_parts(151_147_000, 0)
			.saturating_add(Weight::from_parts(0, 20036))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `ImbueProposals::Projects` (r:1 w:0)
	/// Proof: `ImbueProposals::Projects` (`max_values`: None, `max_size`: Some(36350), added: 38825, mode: `MaxEncodedLen`)
	/// Storage: `ImbueProposals::ProjectsInDispute` (r:1 w:1)
	/// Proof: `ImbueProposals::ProjectsInDispute` (`max_values`: None, `max_size`: Some(61), added: 2536, mode: `MaxEncodedLen`)
	/// Storage: `ImbueDisputes::Disputes` (r:1 w:1)
	/// Proof: `ImbueDisputes::Disputes` (`max_values`: None, `max_size`: Some(6602), added: 9077, mode: `MaxEncodedLen`)
	/// Storage: `ImbueDisputes::DisputesFinaliseOn` (r:1 w:1)
	/// Proof: `ImbueDisputes::DisputesFinaliseOn` (`max_values`: None, `max_size`: Some(221), added: 2696, mode: `MaxEncodedLen`)
	fn raise_dispute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4797`
		//  Estimated: `39815`
		// Minimum execution time: 346_461_000 picoseconds.
		Weight::from_parts(356_015_000, 0)
			.saturating_add(Weight::from_parts(0, 39815))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `ImbueProposals::Projects` (r:1 w:1)
	/// Proof: `ImbueProposals::Projects` (`max_values`: None, `max_size`: Some(36350), added: 38825, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:52 w:52)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn refund() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `11381`
		//  Estimated: `136346`
		// Minimum execution time: 23_947_016_000 picoseconds.
		Weight::from_parts(24_080_686_000, 0)
			.saturating_add(Weight::from_parts(0, 136346))
			.saturating_add(T::DbWeight::get().reads(53))
			.saturating_add(T::DbWeight::get().writes(53))
	}
}
