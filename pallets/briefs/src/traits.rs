
use crate::pallet::*;
use frame_support::{pallet_prelude::*};

pub trait BriefEvolver<Balance, AccountId, BlockNumber> {

    /// Convert a brief into a proposal, the bounty must be fully funded before calling this.
    /// If an Ok is returned the brief pallet will delete the brief from storage as its been converted.
    /// (if using proposals) This function should bypass the usual checks when creating a proposal and
    /// instantiate everything carefully.  
    fn convert_to_proposal(brief_owner: AccountId, bounty: Balance, created_at: BlockNumber, ipfs_hash: IpfsHash) -> Result<()>
}