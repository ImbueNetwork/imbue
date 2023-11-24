use crate::{AccountIdOf, BalanceOf, Contribution, FundingPath, ProposedMilestone, Locality};
use common_types::{CurrencyId, TreasuryOrigin, TreasuryOriginConverter};
use frame_support::{pallet_prelude::*, transactional, PalletId, BoundedBTreeMap};
use frame_system::pallet_prelude::*;
use orml_traits::XcmTransfer;
use orml_xtokens::Error;
use sp_arithmetic::{traits::AtLeast32BitUnsigned, Percent};
use sp_core::H256;
use sp_runtime::traits::AccountIdConversion;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use xcm::latest::{MultiLocation, WeightLimit};

pub trait IntoProposal<AccountId, Balance: AtLeast32BitUnsigned, BlockNumber> {
    type MaximumContributorsPerProject: Get<u32>;
    type MaxMilestonesPerProject: Get<u32>;
    type MaxJuryMembers: Get<u32>;
    /// Convert the propoerties of a project into a project.
    /// This is the main method when wanting to use pallet_proposals and is how one configures a project.
    fn convert_to_proposal(
        currency_id: CurrencyId,
        current_contribution: BoundedBTreeMap<AccountId, Contribution<Balance, BlockNumber>, Self::MaximumContributorsPerProject>,
        brief_hash: H256,
        benificiary: AccountId,
        milestones: BoundedVec<ProposedMilestone, Self::MaxMilestonesPerProject>,
        refund_locations: BoundedVec<(Locality<AccountId>, Percent), Self::MaximumContributorsPerProject>,
        jury: BoundedVec<AccountId, Self::MaxJuryMembers>,
        on_creation_funding: FundingPath,
    ) -> Result<(), DispatchError>;

    /// Convert a btreemap of contributions to multilocations with the Here junction.
    /// Use when the contributors are the refund locations.
    fn convert_contributions_to_refund_locations(
        contributions: &BoundedBTreeMap<AccountId, Contribution<Balance, BlockNumber>, Self::MaximumContributorsPerProject>,
    ) -> BoundedVec<(Locality<AccountId>, Percent), Self::MaximumContributorsPerProject>;
}

pub trait ExternalRefundHandler<AccountId, Balance, CurrencyId> {
    /// Send a message to some destination chain asking to do some reserve asset transfer.
    fn send_refund_message_to_treasury(
        from: AccountId,
        amount: Balance,
        currency: CurrencyId,
        treasury_origin: MultiLocation,
    ) -> Result<(), DispatchError>;
}

#[cfg(feature = "std")]
pub struct MockRefundHandler<T>(T);

#[cfg(feature = "std")]
impl<T: crate::Config> ExternalRefundHandler<AccountIdOf<T>, BalanceOf<T>, CurrencyId>
    for MockRefundHandler<T>
{
    fn send_refund_message_to_treasury(
        _from: AccountIdOf<T>,
        _amount: BalanceOf<T>,
        _currency: CurrencyId,
        _multilocation: MultiLocation
    ) -> Result<(), DispatchError> {
        Ok(())
    }
}

pub struct XcmRefundHandler<T, U>(T, U);
impl<T, U> ExternalRefundHandler<AccountIdOf<T>, T::Balance, CurrencyId> for XcmRefundHandler<T, U>
where
    [u8; 32]: From<BlockNumberFor<T>>,
    T: orml_xtokens::Config,
    U: XcmTransfer<T::AccountId, T::Balance, CurrencyId>,
{
    /// Only used for xcm. Therefore not for briefs and proposals as they use funds which are on imbue.
    #[transactional]
    fn send_refund_message_to_treasury(
        from: T::AccountId,
        amount: T::Balance,
        currency: CurrencyId,
        location: MultiLocation,
    ) -> Result<(), DispatchError> {
        // TODO: dest weight limit. or specify a fee.
        let _ = U::transfer(from, currency, amount, location, WeightLimit::Unlimited)?;
        Ok(())
    }
}
