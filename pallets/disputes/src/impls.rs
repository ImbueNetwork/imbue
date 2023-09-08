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


    // impl<T: Config> DisputeHooks<T::DisputeKey> for Pallet<T> {
    //     // FELIX REVIEW: i think DisputeHooks should be impled where T: pallet_proposals + pallet_refund.
    //     // if you impl it for pallet_dispute then how do we know on pallet_proposals that the dispute is complete?
    //     // call me pls
    //     fn on_dispute_complete(
    //         dispute_key: <T as Config>::DisputeKey,
    //     ) -> Result<(), DispatchError> {
    //         // verifying whether the given dispute exists, if not throwing exception
    //         ensure!(
    //             !Disputes::<T>::contains_key(dispute_key.clone()),
    //             Error::<T>::DisputeDoesNotExist
    //         );
    //         //emitting the dispute as it is completed
    //         //TODO emit the dispute_key
    //         Self::deposit_event(Event::DisputeCompleted);
    //         //removing the dispute once its being completed
    //         Disputes::<T>::remove(dispute_key);
    //         //SHANKAR: How about handling the refund(distrition of the fund) logic here, need to discuss with FELIX
    //         //Also we need to return outcome type correct here?
    //         Ok(())
    //     }

    //     fn on_dispute_cancel(dispute_key: <T as Config>::DisputeKey) -> Result<(), DispatchError> {
    //         // verifying whether the given dispute exists, if not throwing exception
    //         ensure!(
    //             !Disputes::<T>::contains_key(dispute_key.clone()),
    //             Error::<T>::DisputeDoesNotExist
    //         );
    //         //emitting the dispute as it is cancelled
    //         Self::deposit_event(Event::DisputeCancelled);
    //         //removing the dispute once its being cancelled
    //         Disputes::<T>::remove(dispute_key);
    //         //SHANKAR: Confirming incase of cancellation there is no need for any refund logic right?
    //         Ok(())
    //     }
    // }