use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use frame_support::{Deserialize, Serialize};

//#[cfg(feature = "std")]
//use serde::{Deserialize, Serialize};


#[derive(
    Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen, Serialize, Deserialize
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
    Native,
    // Karura KSM
    KSM,
    // Karura Dollar
    AUSD,
    KAR,
    MGX,
    ForeignAsset(ForeignAssetId),
}

pub mod currency_decimals {
    pub const NATIVE: u32 = 12;
    pub const AUSD: u32 = 12;
    pub const KAR: u32 = 12;
    pub const KSM: u32 = 12;
    pub const MGX: u32 = 18;
}

impl Default for CurrencyId {
    fn default() -> Self {
        CurrencyId::Native
    }
}

// A way to generate different currencies from a number.
// Can be used in tests/benchmarks to generate different currencies.
impl From<u32> for CurrencyId {
    fn from(value: u32) -> Self {
        CurrencyId::ForeignAsset(value)
    }
}

pub type ForeignAssetId = u32;

#[derive(
    Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CustomMetadata {
    /// XCM-related metadata.
    /// XCM-related metadata, optional.
    pub xcm: XcmMetadata,
}

#[derive(
    Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct XcmMetadata {
    /// The fee charged for every second that an XCM message takes to execute.
    pub fee_per_second: Option<u128>,
}
