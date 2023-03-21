use crate::pallet::{BriefHash, MilestoneKey};
use proposals::Contribution;
use common_types::CurrencyId;
use std::collections::BTreeMap;

pub trait BriefEvolver<AccountId, Balance, BlockNumber, Milestone, Timestamp> {
    /// Convert a brief into a proposal, the bounty must be fully funded before calling this.
    /// If an Ok is returned the brief pallet will delete the brief from storage as its been converted.
    /// (if using proposals) This function should bypass the usual checks when creating a proposal and
    /// instantiate everything carefully.  
    fn convert_to_proposal(
        brief_owners: Vec<AccountId>,
        bounty_total: Balance,
        currency_id: CurrencyId,
        current_contribution: BTreeMap<AccountId, Contribution<Balance, BlockNumber>>,
        created_at: BlockNumber,
        brief_hash: BriefHash,
        applicant: AccountId,
        milestones: BTreeMap<MilestoneKey, Milestone>
    ) -> Result<(), ()>;
}
