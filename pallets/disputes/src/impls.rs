 use crate::*;

 use traits::{DisputeRaiser,DisputeHooks};

    impl<T: Config> DisputeRaiser<AccountIdOf<T>> for Pallet<T> {
        type DisputeKey = T::DisputeKey;
        type SpecificId = T::SpecificId;
        type MaxReasonLength = <T as Config>::MaxReasonLength;
        type MaxJurySize = <T as Config>::MaxJurySize;
        type MaxSpecifics = <T as Config>::MaxSpecifics;

        fn raise_dispute(
            dispute_key: Self::DisputeKey,
            raised_by: AccountIdOf<T>,
            jury: BoundedVec<AccountIdOf<T>, Self::MaxJurySize>,
            specific_ids: BoundedVec<Self::SpecificId, Self::MaxSpecifics>, 
        ) -> Result<(), DispatchError> {
            crate::Dispute::new(
                dispute_key,
                raised_by,
                jury,
                specifiers,
            )?
        }
    }