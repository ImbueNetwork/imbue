use crate::*;
impl<T: Config> ImmutableIndividualVotes<T> {
    /// Create a new set of individual votes bound to a set of milestone keys.
    /// Instantiates the votes as defaults.
    #[allow(clippy::type_complexity)]
    pub(crate) fn new(
        milestone_keys: BoundedVec<MilestoneKey, T::MaxMilestonesPerProject>,
    ) -> Self {
        let mut outer_votes: BoundedBTreeMap<
            MilestoneKey,
            BoundedBTreeMap<AccountIdOf<T>, bool, T::MaximumContributorsPerProject>,
            T::MaxMilestonesPerProject,
        > = BoundedBTreeMap::new();

        for milestone_key in milestone_keys.iter() {
            let inner_votes: BoundedBTreeMap<
                AccountIdOf<T>,
                bool,
                T::MaximumContributorsPerProject,
            > = BoundedBTreeMap::new();
            // outer_votes and milestone_keys are bounded by the same binding so this will never fail.
            outer_votes
                .try_insert(milestone_key.to_owned(), inner_votes)
                .expect("milestone_keys and outer_votes have been bound by the same binding; qed");
        }

        // Always set as mutable votes for now.
        Self { inner: outer_votes }
    }

    /// Insert the vote from an individual on a milestone.
    pub(crate) fn insert_individual_vote(
        &mut self,
        milestone_key: MilestoneKey,
        account_id: &AccountIdOf<T>,
        vote: bool,
    ) -> Result<(), DispatchError> {
        if let Some(votes) = self.inner.get_mut(&milestone_key) {
            if let Some(_existing_vote) = votes.get_mut(account_id) {
                return Err(Error::<T>::VotesAreImmutable.into());
            } else {
                votes
                    .try_insert(account_id.clone(), vote)
                    .map_err(|_| Error::<T>::TooManyContributions)?;
            }
        } else {
            return Err(Error::<T>::IndividualVoteNotFound.into());
        }

        Ok(())
    }

    /// Clear the votes for a given milestone.
    /// Used when a milestone is submitted.
    /// Skips if the milestone is not found.
    pub(crate) fn clear_milestone_votes(&mut self, milestone_key: MilestoneKey) {
        if let Some(btree) = self.inner.get_mut(&milestone_key) {
            *btree = Default::default()
        }
    }

    /// Take a mutable reference to the inner individual votes item.
    #[allow(dead_code)]
    pub(crate) fn as_mut(&mut self) -> &mut IndividualVotes<T> {
        &mut self.inner
    }
}

impl<T: Config> AsRef<IndividualVotes<T>> for ImmutableIndividualVotes<T> {
    fn as_ref(&self) -> &IndividualVotes<T> {
        &self.inner
    }
}
