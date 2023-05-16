use crate::{Contribution, Event, Milestone, MilestoneKey, Project, ProjectCount, Projects, ProposedMilestone, AccountIdOf, BalanceOf, TimestampOf, pallet};
use common_types::{CurrencyId, FundingType, TreasuryOrigin, TreasuryOriginConverter};
use frame_support::{
    dispatch::EncodeLike, inherent::Vec, pallet_prelude::DispatchError, sp_runtime::Saturating,
    transactional, PalletId,
};
use frame_support::traits::Time;
use frame_system::Config;
use orml_traits::{MultiCurrency, MultiReservableCurrency, XcmTransfer};
use orml_xtokens::Error;

use sp_core::{H256, Get};
use sp_runtime::BoundedBTreeMap;
use sp_runtime::traits::AccountIdConversion;
use xcm::latest::{MultiLocation, WeightLimit};

pub trait IntoProposal<T:Config + pallet::Config> {
    /// Convert a set of milestones into a project.
    /// If an Ok is returned the brief pallet will delete the brief from storage as its been converted.
    /// (if using crate) This function should bypass the usual checks when creating a proposal and
    /// instantiate everything carefully.  
    fn convert_to_proposal(
        currency_id: CurrencyId,
        contributions: BoundedBTreeMap<AccountIdOf<T>, Contribution<BalanceOf<T>, TimestampOf<T>>,T::MaxContributorsPerProject>,
        brief_hash: H256,
        benificiary: AccountIdOf<T>,
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
type ContributionsFor<T> = BoundedBTreeMap<AccountIdOf<T>, Contribution<BalanceOf<T>, TimestampOf<T>>, <T as pallet::Config>::MaxContributorsPerProject>;

impl<T: crate::Config> IntoProposal<T>
    for crate::Pallet<T>
where
    Project<T>: EncodeLike<
        Project<T>,
    >,
{
    /// The caller is used to take the storage deposit from.
    /// With briefs and grants the caller is the beneficiary, so the fee will come from them.
    fn convert_to_proposal(
        currency_id: CurrencyId,
        contributions: ContributionsFor<T>,
        brief_hash: H256,
        benificiary: AccountIdOf<T>,
        proposed_milestones: Vec<ProposedMilestone>,
        funding_type: FundingType,
    ) -> Result<(), DispatchError> {
        let project_key = crate::ProjectCount::<T>::get().saturating_add(1);
        crate::ProjectCount::<T>::put(project_key);

        <<T as crate::Config>::MultiCurrency as MultiReservableCurrency<
                        AccountIdOf<T>,
                    >>::reserve(currency_id, &benificiary, T::ProjectStorageDeposit::get())?;

        let sum_of_contributions = contributions
            .values()
            .fold(Default::default(), |acc: BalanceOf<T>, x| {
                acc.saturating_add(x.value)
            });

        match funding_type {
            FundingType::Proposal | FundingType::Brief => {
                for (acc, cont) in contributions.iter() {
                    let project_account_id = crate::Pallet::<T>::project_account_id(project_key);
                    <<T as crate::Config>::MultiCurrency as MultiReservableCurrency<
                        AccountIdOf<T>,
                    >>::unreserve(currency_id, acc, cont.value);
                    <T as crate::Config>::MultiCurrency::transfer(
                        currency_id,
                        acc,
                        &project_account_id,
                        cont.value,
                    )?;
                }
            }
            FundingType::Treasury(_) => {}
        }

        let mut milestone_key: u32 = 0;
        let mut milestones: BoundedBTreeMap<MilestoneKey, Milestone, T::MaxMilestonesPerProject> = BoundedBTreeMap::new();
        for milestone in proposed_milestones {
            let milestone = Milestone {
                project_key,
                milestone_key,
                percentage_to_unlock: milestone.percentage_to_unlock,
                is_approved: false,
            };
            milestones.try_insert(milestone_key, milestone).expect("TODO: panic message");
            milestone_key = milestone_key.saturating_add(1);
        }

        let project: Project<T> =
            Project {
                milestones,
                contributions: contributions,
                currency_id,
                required_funds: sum_of_contributions,
                withdrawn_funds: 0u32.into(),
                raised_funds: sum_of_contributions,
                initiator: benificiary.clone(),
                created_on: frame_system::Pallet::<T>::block_number(),
                approved_for_funding: true,
                funding_threshold_met: true,
                cancelled: false,
                agreement_hash: brief_hash,
                funding_type,
            };

        Projects::<T>::insert(project_key, project);
        let project_account = Self::project_account_id(project_key);
        ProjectCount::<T>::mutate(|c| *c = c.saturating_add(1));
        Self::deposit_event(Event::ProjectCreated(
            benificiary,
            brief_hash,
            project_key,
            sum_of_contributions,
            currency_id,
            project_account,
        ));

        Ok(())
    }
}

#[cfg(feature = "std")]
pub struct MockRefundHandler<T> {
    phantom_t: sp_std::marker::PhantomData<T>,
}

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

pub struct XcmRefundHandler<T, U> {
    phantom_t: sp_std::marker::PhantomData<T>,
    phantom_u: sp_std::marker::PhantomData<U>,
}

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
            FundingType::Treasury(treasury_origin) => {
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
