use crate::{AccountIdOf, BalanceOf, TimestampOf};
use crate::{Contribution, Milestone, MilestoneKey, Project, Projects, ProposedMilestone};
use common_types::CurrencyId;
use frame_support::dispatch::EncodeLike;
use frame_support::inherent::Vec;
use frame_support::sp_runtime::Saturating;
use orml_traits::MultiCurrency;
use sp_core::H256;
use sp_std::collections::btree_map::BTreeMap;

pub trait BriefEvolver<AccountId, Balance, BlockNumber, TimeStamp> {
    /// Convert a brief into a proposal, the bounty must be fully funded before calling this.
    /// If an Ok is returned the brief pallet will delete the brief from storage as its been converted.
    /// (if using crate) This function should bypass the usual checks when creating a proposal and
    /// instantiate everything carefully.  
    fn convert_to_proposal(
        currency_id: CurrencyId,
        current_contribution: BTreeMap<AccountId, Contribution<Balance, TimeStamp>>,
        brief_hash: H256,
        applicant: AccountId,
        milestones: BTreeMap<MilestoneKey, ProposedMilestone>,
    ) -> Result<(), ()>;
}

type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

impl<T: crate::Config> BriefEvolver<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>>
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
        applicant: AccountIdOf<T>,
        milestones: BTreeMap<MilestoneKey, ProposedMilestone>,
    ) -> Result<(), ()> {
        let project_key = crate::ProjectCount::<T>::get().checked_add(1).ok_or(())?;
        crate::ProjectCount::<T>::put(project_key);

        let sum_of_contributions = contributions
            .values()
            .fold(Default::default(), |acc: BalanceOf<T>, x| {
                acc.saturating_add(x.value)
            });
        let mut project_milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();

        let _ = milestones
            .into_iter()
            .map(|i: (MilestoneKey, ProposedMilestone)| {
                project_milestones.insert(
                    i.0,
                    Milestone {
                        project_key,
                        milestone_key: i.0,
                        percentage_to_unlock: i.1.percentage_to_unlock,
                        is_approved: false,
                    },
                )
            })
            .collect::<Vec<_>>();

        let project: Project<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>> =
            Project {
                milestones: project_milestones,
                contributions: contributions,
                currency_id,
                required_funds: sum_of_contributions,
                withdrawn_funds: 0u32.into(),
                raised_funds: sum_of_contributions,
                initiator: applicant,
                create_block_number: frame_system::Pallet::<T>::block_number(),
                approved_for_funding: true,
                funding_threshold_met: true,
                cancelled: false,
                agreement_hash: brief_hash,
                // Maybe we dont need this new field because we have create_block_number
                work_started_at: Some(frame_system::Pallet::<T>::block_number()),
            };

        Projects::<T>::insert(project_key, project);

        Ok(())
    }
}
