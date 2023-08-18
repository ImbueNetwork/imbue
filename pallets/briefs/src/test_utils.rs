use crate::pallet::BriefHash;
use sp_core::H256;
pub fn gen_hash(seed: u8) -> BriefHash {
    H256::from([seed; 32])
}
