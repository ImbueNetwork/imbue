use common_types::CurrencyId;
use proposals::{AccountIdOf, BalanceOf, TimestampOf};
use proposals::{Project, Projects, Contribution, MilestoneKey, Milestone};
use sp_std::collections::btree_map::BTreeMap;
use sp_core::H256;
use frame_support::inherent::Vec;
use frame_support::dispatch::EncodeLike;
use frame_support::sp_runtime::Saturating;
use orml_traits::MultiCurrency;
use crate::pallet::{BriefMilestone, BriefHash};


pub trait BriefEvolver<AccountId, Balance, BlockNumber, Milestone, TimeStamp> {
    /// Convert a brief into a proposal, the bounty must be fully funded before calling this.
    /// If an Ok is returned the brief pallet will delete the brief from storage as its been converted.
    /// (if using proposals) This function should bypass the usual checks when creating a proposal and
    /// instantiate everything carefully.  
    fn convert_to_proposal(
        brief_owners: Vec<AccountId>,
        bounty_total: Balance,
        currency_id: CurrencyId,
        current_contribution: BTreeMap<AccountId, Contribution<Balance, TimeStamp>>,
        created_at: BlockNumber,
        brief_hash: H256,
        applicant: AccountId,
        milestones: BTreeMap<MilestoneKey, Milestone>
    ) -> Result<(), ()>;
}

type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

impl<T: proposals::Config> BriefEvolver<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, BriefMilestone, TimestampOf<T>> for crate::Pallet<T>
where
    Project<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>>: EncodeLike<
        Project<
            <T as frame_system::Config>::AccountId,
            <<T as proposals::Config>::MultiCurrency as MultiCurrency<
                <T as frame_system::Config>::AccountId,
            >>::Balance,
            <T as frame_system::Config>::BlockNumber,
            <T as pallet_timestamp::Config>::Moment,
        >,
    >,
{
    fn convert_to_proposal(
        brief_owners: Vec<AccountIdOf<T>>,
        bounty_total: BalanceOf<T>,
        currency_id: CurrencyId,
        contributions: BTreeMap<AccountIdOf<T>, Contribution<BalanceOf<T>, TimestampOf<T>>>,
        created_at: BlockNumberFor<T>,
        brief_hash: H256,
        applicant: AccountIdOf<T>,
        milestones: BTreeMap<MilestoneKey, BriefMilestone>
    ) -> Result<(), ()> {

        let project_key = proposals::ProjectCount::<T>::get().checked_add(1).ok_or(())?;
        proposals::ProjectCount::<T>::put(project_key);

        let mut project_milestones: BTreeMap<MilestoneKey, Milestone> = BTreeMap::new();
        let _ = milestones.into_values().map(|m| {
            project_milestones.insert(m.milestone_key, 
                proposals::Milestone {
                    project_key: project_key,
                    milestone_key: m.milestone_key,
                    name: m.name.into(),
                    percentage_to_unlock: m.percentage_to_unlock,
                    is_approved: false,
                }
            )
        }).collect::<Vec<_>>();
        
        let sum_of_contributions = contributions.values().fold(Default::default(), |acc: BalanceOf<T>, x| acc.saturating_add(x.value));

        let project: Project<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, TimestampOf<T>> = Project {
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

