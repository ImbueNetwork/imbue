
use crate::pallet::*;
use frame_support::{pallet_prelude::*};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use common_types::CurrencyId;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_support::dispatch::fmt::Debug;
use sp_runtime::traits::{MaybeDisplay, AtLeast32BitUnsigned, Bounded, Hash};
use sp_std::str::FromStr;
use codec::FullCodec;

use frame_support::traits::tokens::Balance;

pub trait BriefEvolver<AccountId, Balance, BlockNumber>
{
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
        applicant: AccountId) -> Result<(), ()>;

}