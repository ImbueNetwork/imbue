
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions::*, MultiLocation, NetworkId};


/// A wrapper around
pub trait TreasuryOriginConverter<AccountId: Into<[u8; 32]>> {
    fn get_multi_location(&self, recipiant: AccountId) -> Result<MultiLocation, TreasuryOriginError>;
}

impl<AccountId: Into<[u8; 32]>> TreasuryOriginConverter<AccountId> for TreasuryOrigin {
    fn get_multi_location(&self, recipiant: AccountId) -> Result<MultiLocation, TreasuryOriginError> {
        match &self {
            TreasuryOrigin::Kusama => {
                Ok
                (
                    MultiLocation::new(
                        1,
                        X1(Junction::AccountId32 {
                            id: recipiant.into(),
                            network: Some(NetworkId::Kusama),
                        })
                    )
                )
            },
            TreasuryOrigin::Imbue => {
                Ok(Default::default())
            },
            _ => {
                Err(TreasuryOriginError::NetworkUnsupported)
            }
        }
    }
}   


#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TreasuryOriginError {
    GenericError,
    NetworkUnsupported,
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum FundingType {
    Proposal,
    Brief,
    Treasury(TreasuryOrigin),
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TreasuryOrigin {
    Kusama,
    Imbue,
    Karura,
}
