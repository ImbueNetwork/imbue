use crate::*;
use frame_support::BoundedVec;
use sp_runtime::DispatchError;

use crate::pallet::{AccountIdOf, Config, Dispute};
use crate::traits::{DisputeRaiser, DisputeHooks};

impl<T: Config> DisputeRaiser<AccountIdOf<T>> for Pallet<T> {
    type DisputeKey = T::DisputeKey;
    type SpecificId = T::SpecificId;
    type MaxJurySize = <T as Config>::MaxJurySize;
    type MaxSpecifics = <T as Config>::MaxSpecifics;

    /// Public interface for Dispute::new()
    fn raise_dispute(
        dispute_key: Self::DisputeKey,
        raised_by: AccountIdOf<T>,
        jury: BoundedVec<AccountIdOf<T>, Self::MaxJurySize>,
        specifiers: BoundedVec<Self::SpecificId, Self::MaxSpecifics>,
    ) -> Result<(), DispatchError> {
        // https://github.com/ImbueNetwork/imbue/issues/270
        if jury.len() == 1usize {
            let _ = T::DisputeHooks::on_dispute_complete(dispute_key, specifiers.to_vec(), DisputeResult::Success);
            return Ok(())
        }

        Dispute::<T>::new(dispute_key, raised_by, jury, specifiers)?;
        Ok(())
    }
}
