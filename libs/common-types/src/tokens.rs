use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use common_traits::TokenMetadata;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

#[derive(
    Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
    Native,
    // Karura KSM
    KSM,
    // Karura Dollar
    KUSD,
}

impl TokenMetadata for CurrencyId {
    fn name(&self) -> Vec<u8> {
        match self {
            CurrencyId::Native => b"Native currency".to_vec(),
            CurrencyId::KUSD => b"Karura Dollar".to_vec(),
            CurrencyId::KSM => b"Kusama".to_vec(),
        }
    }

    fn symbol(&self) -> Vec<u8> {
        match self {
            CurrencyId::Native => b"IMBU".to_vec(),
            CurrencyId::KUSD => b"KUSD".to_vec(),
            CurrencyId::KSM => b"KSM".to_vec(),
        }
    }

    fn decimals(&self) -> u8 {
        match self {
            CurrencyId::Native => 18,
            CurrencyId::KUSD | CurrencyId::KSM => 12,
        }
    }
}
