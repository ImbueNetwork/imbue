
//! Autogenerated weights for `pallet_fellowship`
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
// pallet-fellowship
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

/// Weight functions for `pallet_fellowship`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> crate::WeightInfoT for WeightInfo<T> {
	/// Storage: `ImbueFellowship::Roles` (r:1 w:1)
	/// Proof: `ImbueFellowship::Roles` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::FellowToVetter` (r:0 w:1)
	/// Proof: `ImbueFellowship::FellowToVetter` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::FellowshipReserves` (r:0 w:1)
	/// Proof: `ImbueFellowship::FellowshipReserves` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	fn add_to_fellowship() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `3593`
		// Minimum execution time: 297_927_000 picoseconds.
		Weight::from_parts(299_772_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `ImbueFellowship::Roles` (r:1 w:1)
	/// Proof: `ImbueFellowship::Roles` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	fn force_add_fellowship() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `3516`
		// Minimum execution time: 153_043_000 picoseconds.
		Weight::from_parts(154_780_000, 0)
			.saturating_add(Weight::from_parts(0, 3516))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `ImbueFellowship::Roles` (r:1 w:1)
	/// Proof: `ImbueFellowship::Roles` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::PendingFellows` (r:1 w:1)
	/// Proof: `ImbueFellowship::PendingFellows` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::FellowshipReserves` (r:1 w:0)
	/// Proof: `ImbueFellowship::FellowshipReserves` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::FellowToVetter` (r:0 w:1)
	/// Proof: `ImbueFellowship::FellowToVetter` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	fn leave_fellowship() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `410`
		//  Estimated: `3593`
		// Minimum execution time: 463_315_000 picoseconds.
		Weight::from_parts(479_038_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `ImbueFellowship::Roles` (r:1 w:1)
	/// Proof: `ImbueFellowship::Roles` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::PendingFellows` (r:1 w:1)
	/// Proof: `ImbueFellowship::PendingFellows` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::FellowshipReserves` (r:1 w:0)
	/// Proof: `ImbueFellowship::FellowshipReserves` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::FellowToVetter` (r:0 w:1)
	/// Proof: `ImbueFellowship::FellowToVetter` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	fn force_remove_and_slash_fellowship() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `513`
		//  Estimated: `6196`
		// Minimum execution time: 894_392_000 picoseconds.
		Weight::from_parts(915_373_000, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `ImbueFellowship::Roles` (r:2 w:0)
	/// Proof: `ImbueFellowship::Roles` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::ShortlistRound` (r:1 w:0)
	/// Proof: `ImbueFellowship::ShortlistRound` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::CandidateShortlist` (r:1 w:1)
	/// Proof: `ImbueFellowship::CandidateShortlist` (`max_values`: None, `max_size`: Some(3421), added: 5896, mode: `MaxEncodedLen`)
	fn add_candidate_to_shortlist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `289`
		//  Estimated: `6886`
		// Minimum execution time: 270_206_000 picoseconds.
		Weight::from_parts(278_894_000, 0)
			.saturating_add(Weight::from_parts(0, 6886))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `ImbueFellowship::Roles` (r:1 w:0)
	/// Proof: `ImbueFellowship::Roles` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::ShortlistRound` (r:1 w:0)
	/// Proof: `ImbueFellowship::ShortlistRound` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::CandidateShortlist` (r:1 w:1)
	/// Proof: `ImbueFellowship::CandidateShortlist` (`max_values`: None, `max_size`: Some(3421), added: 5896, mode: `MaxEncodedLen`)
	fn remove_candidate_from_shortlist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `299`
		//  Estimated: `6886`
		// Minimum execution time: 205_598_000 picoseconds.
		Weight::from_parts(207_340_000, 0)
			.saturating_add(Weight::from_parts(0, 6886))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `ImbueFellowship::PendingFellows` (r:1 w:1)
	/// Proof: `ImbueFellowship::PendingFellows` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::Roles` (r:0 w:1)
	/// Proof: `ImbueFellowship::Roles` (`max_values`: None, `max_size`: Some(51), added: 2526, mode: `MaxEncodedLen`)
	/// Storage: `ImbueFellowship::FellowshipReserves` (r:0 w:1)
	/// Proof: `ImbueFellowship::FellowshipReserves` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	fn pay_deposit_to_remove_pending_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `259`
		//  Estimated: `3593`
		// Minimum execution time: 392_591_000 picoseconds.
		Weight::from_parts(401_207_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}
