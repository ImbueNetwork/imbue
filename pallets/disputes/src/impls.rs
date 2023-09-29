use crate::*;
use frame_support::BoundedVec;
use sp_runtime::DispatchError;

use crate::pallet::{AccountIdOf, Config, Dispute};
use traits::DisputeRaiser;

impl<T: Config> DisputeRaiser<AccountIdOf<T>> for Pallet<T> {
    type DisputeKey = T::DisputeKey;
    type SpecificId = T::SpecificId;
    type MaxReasonLength = <T as Config>::MaxReasonLength;
    type MaxJurySize = <T as Config>::MaxJurySize;
    type MaxSpecifics = <T as Config>::MaxSpecifics;

    /// Public interface for Dispute::new()
    fn raise_dispute(
        dispute_key: Self::DisputeKey,
        raised_by: AccountIdOf<T>,
        jury: BoundedVec<AccountIdOf<T>, Self::MaxJurySize>,
        specifiers: BoundedVec<Self::SpecificId, Self::MaxSpecifics>,
    ) -> Result<(), DispatchError> {
        Dispute::<T>::new(dispute_key, raised_by, jury, specifiers)?;
        Self::deposit_event(Event::<T>::DisputeRaised { dispute_key });
        Ok(())
    }
}
