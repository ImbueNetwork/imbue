use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use frame_support::traits::ConstU32;
use serde::{Deserialize, Serialize};
use sp_runtime::BoundedVec;
#[derive(
    Clone,
    Copy,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Debug,
    Encode,
    Decode,
    TypeInfo,
    MaxEncodedLen,
    Default,
    Serialize,
    Deserialize,
)]
pub enum CurrencyId {
    #[default]
    Native,
    // Karura KSM
    KSM,
    // Karura Dollar
    AUSD,
    KAR,
    MGX,
    ForeignAsset(ForeignAssetId),
}

#[derive(
    Clone,
    Copy,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Debug,
    Encode,
    Decode,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub enum ForeignAssetId {
    ETH,
    USDT,
    ADA,
}

#[derive(
    Clone,
    // Copy,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Debug,
    Encode,
    Decode,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
/// The foreign owned account describes the chain
pub enum ForeignOwnedAccount {
    TRON([u8; 22]),
    ETH([u8; 20]),
    ADA(BoundedVec<u8, ConstU32<104>>),
}

impl ForeignOwnedAccount {
    /// Here we can define which currencies per network we support
    /// For example when given a TRON account we can use this to see if the account
    /// and the currency are compatible.
    pub fn ensure_supported_currency(&self, currency: CurrencyId) -> bool {
        match currency {
            CurrencyId::Native => false,
            CurrencyId::KSM => false,
            CurrencyId::AUSD => false,
            CurrencyId::KAR => false,
            CurrencyId::MGX => false,
            CurrencyId::ForeignAsset(asset) => match &self {
                ForeignOwnedAccount::TRON(_) => match asset {
                    ForeignAssetId::USDT => true,
                    default => false,
                },
                ForeignOwnedAccount::ETH(_) => match asset {
                    ForeignAssetId::ETH => true,
                    ForeignAssetId::USDT => true,
                    default => false,
                },
                ForeignOwnedAccount::ADA(_) => match asset {
                    ForeignAssetId::ADA => true,
                    default => false,
                },
            },
        }
    }
    #[cfg(feature = "runtime-benchmarks")]
    pub fn get_supported_currency_eoa_combo() -> (ForeignOwnedAccount, CurrencyId) {
        (
            ForeignOwnedAccount::ETH(Default::default()),
            CurrencyId::ForeignAsset(ForeignAssetId::ETH),
        )
    }
}

pub mod currency_decimals {
    pub const NATIVE: u32 = 12;
    pub const AUSD: u32 = 12;
    pub const KAR: u32 = 12;
    pub const KSM: u32 = 12;
    pub const MGX: u32 = 18;
}

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
