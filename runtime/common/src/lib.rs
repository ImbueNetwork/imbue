// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.


#![cfg_attr(not(feature = "std"), no_std)]

pub use constants::*;
pub use types::*;


/// Common types for all runtimes
pub mod types {
	use frame_support::traits::EnsureOneOf;
	use frame_system::EnsureRoot;
	use sp_runtime::traits::{BlakeTwo256, IdentifyAccount, Verify};
	use sp_std::vec::Vec;

	pub type EnsureRootOr<O> = EnsureOneOf<EnsureRoot<AccountId>, O>;

	/// An index to a block.
	pub type BlockNumber = u32;

	/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
	pub type Signature = sp_runtime::MultiSignature;

	/// Some way of identifying an account on the chain. We intentionally make it equivalent
	/// to the public key of our transaction signing scheme.
	pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
	/// never know...
	pub type AccountIndex = u32;

	/// The address format for describing accounts.
	pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

	/// Balance of an account.
	pub type Balance = u128;

	/// Index of a transaction in the chain.
	pub type Index = u32;

	/// A hash of some data used by the chain.
	pub type Hash = sp_core::H256;

	/// Block header type as expected by this runtime.
	pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;


	/// Aura consensus authority.
	pub type AuraId = sp_consensus_aura::sr25519::AuthorityId;

	/// Moment type
	pub type Moment = u64;

	// A vector of bytes, conveniently named like it is in Solidity.
	pub type Bytes = Vec<u8>;

	// A 32 bytes fixed-size array.
	pub type Bytes32 = FixedArray<u8, 32>;

	// Fixed-size array of given typed elements.
	pub type FixedArray<T, const S: usize> = [T; S];

	// A cryptographic salt to be combined with a value before hashing.
	pub type Salt = FixedArray<u8, 32>;
}

pub mod currency {
	use super::types::Balance;

	pub const MICRO_IMBU: Balance = 1_000_000_000_000; // 10−6 	0.000001
	pub const MILLI_IMBU: Balance = 1_000 * MICRO_IMBU; // 10−3 	0.001
	pub const CENTI_IMBU: Balance = 10 * MILLI_IMBU; // 10−2 	0.01
	pub const IMBU: Balance = 100 * CENTI_IMBU;

	pub const EXISTENTIAL_DEPOSIT: Balance = 1 * MICRO_IMBU;

	/// Minimum vesting amount, in IMBU/PCHU
	pub const MIN_VESTING: Balance = 10;

	/// Additional fee charged when moving native tokens to target chains (in IMBUs).
	pub const NATIVE_TOKEN_TRANSFER_FEE: Balance = 2000 * IMBU;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		// map to 1/10 of what the kusama relay chain charges (v9020)
		(items as Balance * 2_000 * CENTI_IMBU + (bytes as Balance) * 100 * MILLI_IMBU) / 10
	}

}

/// Common constants for all runtimes
pub mod constants {
	use super::types::BlockNumber;
	use frame_support::weights::{constants::WEIGHT_PER_SECOND, Weight};
	use sp_runtime::Perbill;

	/// This determines the average expected block time that we are targeting. Blocks will be
	/// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
	/// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
	/// slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: u64 = 6000;
	pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	/// Milliseconds per day
	pub const MILLISECS_PER_DAY: u64 = 86400000;

	/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
	/// used to limit the maximal weight of a single extrinsic.
	pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);
	/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
	/// Operational  extrinsics.
	pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

	/// We allow for 0.5 seconds of compute with a 6 second average block time.
	pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
}
