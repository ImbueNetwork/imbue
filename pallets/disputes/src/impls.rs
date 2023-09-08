 use crate::*;

 use traits::{DisputeRaiser,DisputeHooks};

    impl<T: Config> DisputeRaiser<AccountIdOf<T>> for Pallet<T> {
        type DisputeKey = T::DisputeKey;
        type MaxReasonLength = <T as Config>::MaxReasonLength;
        type MaxJurySize = <T as Config>::MaxJurySize;

        fn raise_dispute(
            dispute_key: Self::DisputeKey,
            raised_by: AccountIdOf<T>,
            fund_account: AccountIdOf<T>,
            reason: BoundedVec<u8, Self::MaxReasonLength>,
            jury: BoundedVec<AccountIdOf<T>, Self::MaxJurySize>,
        ) -> Result<(), DispatchError> {

            // creating the struct with the passed information and initializing vote as 0 initially
            let dispute: Dispute<T> = Dispute {
                raised_by: raised_by.clone(),
                fund_account,
                votes: Vote::Refund(RefundVote {
                    to_initiator: 0,
                    to_refund: 0,
                }),
                reason,
                jury,
            };

            ensure!(
                !Disputes::<T>::contains_key(dispute_key.clone()),
                Error::<T>::DisputeAlreadyExists
            );

            //storing the raised dispute inside the disputes storage
            Disputes::<T>::insert(dispute_key, dispute);

            // Raise Event
            //SHANKAR if want to add more information while raising a dispute like returning the whole dispute struct to
            //get some more info about what has been raised or so on?
            //FELIX REVIEW Only add information to events that you are certain youre gonna use on the app.
            //each event is stored on chain so it would bloat the chain if we started returning whole structs.
            Self::deposit_event(Event::DisputeRaised { who: raised_by });

            Ok(())
        }
    }