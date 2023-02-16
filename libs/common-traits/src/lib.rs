// Ensure we're `no_std` when compiling for WebAssembly.

//! # A common trait for imbue
//!
//! This crate provides some common traits used imbue.
//! # Reward trait
//! The trait does assume, that any call of reward has been
//! checked for validity. I.e. there are not validation checks
//! provided by the trait.

// Ensure we're `no_std` when compiling for WebAssembly.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{Codec, DispatchResultWithPostInfo};
use frame_support::Parameter;
use sp_runtime::traits::{
    AtLeast32BitUnsigned, Bounded, MaybeDisplay, MaybeSerialize,
    MaybeSerializeDeserialize, Member, Zero,
};
use sp_std::fmt::Debug;
use sp_std::hash::Hash;
use sp_std::str::FromStr;
use sp_std::vec::Vec;
/// A trait used for loosely coupling the claim pallet with a reward mechanism.
///
/// ## Overview
/// The crowdloan reward mechanism is separated from the crowdloan claiming process, the latter
/// being generic, acting as a kind of proxy to the rewarding mechanism, that is specific to
/// to each crowdloan campaign. The aim of this pallet is to ensure that a claim for a reward
/// payout is well-formed, checking for replay attacks, spams or invalid claim (e.g. unknown
/// contributor, exceeding reward amount, ...).
/// See the [`crowdloan-reward`] pallet, that implements a reward mechanism with vesting, for
/// instance.
pub trait Reward {
    /// The account from the parachain, that the claimer provided in her/his transaction.
    type ParachainAccountId: Debug
        + Default
        + MaybeSerialize
        + MaybeSerializeDeserialize
        + Member
        + Ord
        + Parameter;

    /// The contribution amount in relay chain tokens.
    type ContributionAmount: AtLeast32BitUnsigned
        + Codec
        + Copy
        + Debug
        + Default
        + MaybeSerializeDeserialize
        + Member
        + Parameter
        + Zero;

    /// Block number type used by the runtime
    type BlockNumber: AtLeast32BitUnsigned
        + Bounded
        + Copy
        + Debug
        + Default
        + FromStr
        + Hash
        + MaybeDisplay
        + MaybeSerializeDeserialize
        + Member
        + Parameter;

    /// Rewarding function that is invoked from the claim pallet.
    ///
    /// If this function returns successfully, any subsequent claim of the same claimer will be
    /// rejected by the claim module.
    fn reward(
        who: Self::ParachainAccountId,
        contribution: Self::ContributionAmount,
    ) -> DispatchResultWithPostInfo;
}

/// A trait used to convert a type to BigEndian format
pub trait BigEndian<T> {
    fn to_big_endian(&self) -> T;
}

/// A trait that Assets or Tokens can implement so that pallets
/// can easily use the trait `InspectMetadata` with them.
pub trait TokenMetadata {
    fn name(&self) -> Vec<u8>;

    fn symbol(&self) -> Vec<u8>;

    fn decimals(&self) -> u8;
}
