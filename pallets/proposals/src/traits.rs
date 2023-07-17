use crate::{AccountIdOf, BalanceOf, Contribution, ProposedMilestone};
use common_types::{CurrencyId, FundingType, TreasuryOrigin, TreasuryOriginConverter};
use frame_support::{inherent::Vec, pallet_prelude::DispatchError, transactional, PalletId};
use orml_traits::XcmTransfer;
use orml_xtokens::Error;

use sp_core::H256;
use sp_runtime::traits::AccountIdConversion;
use sp_std::collections::btree_map::BTreeMap;
use xcm::latest::{MultiLocation, WeightLimit};

pub trait IntoProposal<AccountId, Balance, BlockNumber> {
    /// Convert a set of milestones into a proposal, the bounty must be fully funded before calling this.
    /// If an Ok is returned the brief pallet will delete the brief from storage as its been converted.
    /// (if using crate) This function should bypass the usual checks when creating a proposal and
    /// instantiate everything carefully.
    // TODO: Generic over currencyId: https://github.com/ImbueNetwork/imbue/issues/135
    fn convert_to_proposal(
        currency_id: CurrencyId,
        current_contribution: BTreeMap<AccountId, Contribution<Balance, BlockNumber>>,
        brief_hash: H256,
        benificiary: AccountId,
        milestones: Vec<ProposedMilestone>,
        funding_type: FundingType,
    ) -> Result<(), DispatchError>;
}

pub trait RefundHandler<AccountId, Balance, CurrencyId> {
    /// Send a message to some destination chain asking to do some reserve asset transfer.
    /// The multilocation is defined by the FundingType.
    /// see FundingType and TreasuryOrigin.
    fn send_refund_message_to_treasury(
        from: AccountId,
        amount: Balance,
        currency: CurrencyId,
        funding_type: FundingType,
    ) -> Result<(), DispatchError>;
    fn get_treasury_account_id(treasury_origin: TreasuryOrigin)
        -> Result<AccountId, DispatchError>;
}


// Some implementations used in Imbue of the traits above.
type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;
// For test purposes
impl<T: crate::Config> IntoProposal<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>> for T {
    fn convert_to_proposal(
        _currency_id: CurrencyId,
        _contributions: BTreeMap<AccountIdOf<T>, Contribution<BalanceOf<T>, BlockNumberFor<T>>>,
        _brief_hash: H256,
        _benificiary: AccountIdOf<T>,
        _proposed_milestones: Vec<ProposedMilestone>,
        _funding_type: FundingType,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
}

#[cfg(feature = "std")]
pub struct MockRefundHandler<T>(T);

#[cfg(feature = "std")]
impl<T: crate::Config> RefundHandler<AccountIdOf<T>, BalanceOf<T>, CurrencyId>
    for MockRefundHandler<T>
{
    fn send_refund_message_to_treasury(
        _from: AccountIdOf<T>,
        _amount: BalanceOf<T>,
        _currency: CurrencyId,
        _funding_type: FundingType,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
    fn get_treasury_account_id(
        _treasury_account: TreasuryOrigin,
    ) -> Result<AccountIdOf<T>, DispatchError> {
        Ok(PalletId(*b"py/trsry").into_account_truncating())
    }
}

pub struct XcmRefundHandler<T, U>(T, U);

impl<T, U> RefundHandler<AccountIdOf<T>, T::Balance, CurrencyId> for XcmRefundHandler<T, U>
where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    T: orml_xtokens::Config,
    U: XcmTransfer<T::AccountId, T::Balance, CurrencyId>,
{
    /// Only used for xcm. Therefore not for briefs and proposals as they use funds which are on imbue.
    #[transactional]
    fn send_refund_message_to_treasury(
        from: T::AccountId,
        amount: T::Balance,
        currency: CurrencyId,
        funding_type: FundingType,
    ) -> Result<(), DispatchError> {
        match funding_type {
            FundingType::Grant(treasury_origin) => {
                let beneficiary: AccountIdOf<T> = Self::get_treasury_account_id(treasury_origin)?;
                let location: MultiLocation = treasury_origin
                    .get_multi_location(beneficiary)
                    .map_err(|_| Error::<T>::InvalidDest)?;

                // TODO: dest weight limit. or specify a fee.
                let _ = U::transfer(from, currency, amount, location, WeightLimit::Unlimited)?;
                Ok(())
            }
            _ => Err(Error::<T>::InvalidDest.into()),
        }
    }
    fn get_treasury_account_id(
        treasury_origin: TreasuryOrigin,
    ) -> Result<AccountIdOf<T>, DispatchError> {
        match treasury_origin {
            TreasuryOrigin::Kusama => {
                // TODO: make this dynamic so its always correct.
                Ok(PalletId(*b"py/trsry").into_account_truncating())
            }
            _ => {
                // At the moment just supporting kusama but allow this instead of a panic
                Ok(PalletId(*b"py/trsry").into_account_truncating())
            }
        }
    }
}
