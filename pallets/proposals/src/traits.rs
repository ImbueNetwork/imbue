use crate::{AccountIdOf, BalanceOf, TimestampOf};
use crate::{
    Contribution, Event, Milestone, MilestoneKey, Project, ProjectCount, Projects,
    ProposedMilestone,
};
use common_types::{CurrencyId, FundingType};
use frame_support::dispatch::EncodeLike;
use frame_support::inherent::Vec;
use frame_support::sp_runtime::Saturating;
use orml_traits::MultiCurrency;
use sp_core::H256;
use sp_std::collections::btree_map::BTreeMap;

pub trait IntoProposal<AccountId, Balance, BlockNumber, TimeStamp> {
    /// Convert a set of milestones into a proposal, the bounty must be fully funded before calling this.
    /// If an Ok is returned the brief pallet will delete the brief from storage as its been converted.
    /// (if using crate) This function should bypass the usual checks when creating a proposal and
    /// instantiate everything carefully.  
    fn convert_to_proposal(
        currency_id: CurrencyId,
        current_contribution: BTreeMap<AccountId, Contribution<Balance, TimeStamp>>,
        brief_hash: H256,
        benificiary: AccountId,
        milestones: Vec<ProposedMilestone>,
        funding_type: FundingType,
    ) -> Result<(), ()>;
}

type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

impl<T: crate::Config> IntoProposal<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>>
    for crate::Pallet<T>
where
    Project<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>>: EncodeLike<
        Project<
            <T as frame_system::Config>::AccountId,
            <<T as crate::Config>::MultiCurrency as MultiCurrency<
                <T as frame_system::Config>::AccountId,
            >>::Balance,
            <T as frame_system::Config>::BlockNumber,
            <T as pallet_timestamp::Config>::Moment,
        >,
    >,
{
    fn convert_to_proposal(
        currency_id: CurrencyId,
        contributions: BTreeMap<AccountIdOf<T>, Contribution<BalanceOf<T>, TimestampOf<T>>>,
        brief_hash: H256,
        benificiary: AccountIdOf<T>,
        proposed_milestones: Vec<ProposedMilestone>,
        funding_type: FundingType,
    ) -> Result<(), ()> {
        let project_key = crate::ProjectCount::<T>::get().checked_add(1).ok_or(())?;
        crate::ProjectCount::<T>::put(project_key);

        let sum_of_contributions = contributions
            .values()
            .fold(Default::default(), |acc: BalanceOf<T>, x| {
                acc.saturating_add(x.value)
            });

        let mut milestone_key: u32 = 0;
        let mut milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();
        for milestone in proposed_milestones {
            let milestone = Milestone {
                project_key,
                milestone_key,
                percentage_to_unlock: milestone.percentage_to_unlock,
                is_approved: false,
            };
            milestones.insert(milestone_key, milestone);
            milestone_key = milestone_key.checked_add(1).unwrap_or(0);
        }

        let project: Project<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>> =
            Project {
                milestones,
                contributions,
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
        ProjectCount::<T>::mutate(|c| *c += 1);
        Self::deposit_event(Event::ProjectCreated(
            benificiary,
            brief_hash,
            project_key,
            sum_of_contributions,
            currency_id,
        ));

        Ok(())
    }
}
