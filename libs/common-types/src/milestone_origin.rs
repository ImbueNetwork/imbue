use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{sp_runtime::traits::AccountIdConversion, PalletId};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions::*, MultiLocation};

/// A wrapper around
pub trait TreasuryOriginConverter {
    fn get_multi_location(&self) -> Result<MultiLocation, TreasuryOriginError>;
}

impl TreasuryOriginConverter for TreasuryOrigin {
    fn get_multi_location(&self) -> Result<MultiLocation, TreasuryOriginError> {
        match &self {
            TreasuryOrigin::Kusama => Ok(MultiLocation::new(
                1,
                X1(Junction::AccountId32 {
                    id: PalletId(*b"py/trsry").into_account_truncating(),
                    network: None,
                }),
            )),
            TreasuryOrigin::Imbue => Ok(MultiLocation::new(
                0,
                X1(Junction::AccountId32 {
                    id: PalletId(*b"py/trsry").into_account_truncating(),
                    network: None,
                }),
            )),
            _ => Err(TreasuryOriginError::NetworkUnsupported),
        }
    }
}

#[derive(
    Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
pub enum TreasuryOriginError {
    GenericError,
    NetworkUnsupported,
}

#[derive(
    Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
pub enum TreasuryOrigin {
    Kusama,
    Imbue,
    Karura,
}
