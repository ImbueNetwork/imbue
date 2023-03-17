use crate::pallet::*;
use common_types::CurrencyId;

pub trait BriefEvolver<AccountId, Balance, BlockNumber> {
    /// Convert a brief into a proposal, the bounty must be fully funded before calling this.
    /// If an Ok is returned the brief pallet will delete the brief from storage as its been converted.
    /// (if using proposals) This function should bypass the usual checks when creating a proposal and
    /// instantiate everything carefully.  
    fn convert_to_proposal(
        brief_owners: Vec<AccountId>,
        bounty_total: Balance,
        currency_id: CurrencyId,
        current_contribution: Balance,
        created_at: BlockNumber,
        ipfs_hash: IpfsHash,
        applicant: AccountId,
    ) -> Result<(), ()>;
}