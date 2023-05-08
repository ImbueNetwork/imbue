use crate as pallet_grants;
use sp_core::H256;
pub fn gen_grant_id(seed: u8) -> pallet_grants::GrantId {
    H256::from([seed; 32])
}

