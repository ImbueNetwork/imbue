use std::ptr::null;
use crate::impls::*;
use crate::traits::*;
use crate::*;
use crate::{mock::*, Error, Event};
use common_types::CurrencyId;
use frame_support::{assert_noop, assert_ok, once_cell::sync::Lazy, BoundedBTreeMap, traits::Hooks};
use frame_system::Pallet as System;
use orml_tokens::Error as TokensError;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_core::sr25519::Public;
use sp_runtime::{traits::BadOrigin, DispatchError, Saturating, BoundedVec};
use sp_arithmetic::traits::One;

pub fn run_to_block<T: Config>(n: T::BlockNumber)
    where T::BlockNumber: Into<u64>
{
    loop {
        let mut block: T::BlockNumber = frame_system::Pallet::<T>::block_number();
        if block >= n {
            break;
        }
        block = block.saturating_add(<T::BlockNumber as One>::one());
        frame_system::Pallet::<T>::set_block_number(block);
        frame_system::Pallet::<T>::on_initialize(block);
    }
}


#[test]
fn test_create_a_dispute_get_and_check_insertion_within_the_storage() {
    const DISPUTE_KEY: u32 = 10;
    new_test_ext().execute_with(|| {
        Dispute::<Test>::new(DISPUTE_KEY, **&ALICE, BoundedVec::default(), BoundedVec::default()).expect("TODO: panic message");
        assert!(PalletDisputes::disputes(DISPUTE_KEY).is_some());
        assert_eq!(1, PalletDisputes::disputes(DISPUTE_KEY).iter().count());
    });
}

#[test]
fn test_trying_to_insert_create_a_dispute_with_existing_key() {
    new_test_ext().execute_with(|| {
        Dispute::<Test>::new(1, *ALICE, BoundedVec::default(), BoundedVec::default()).expect("Creation Failed");
        assert_noop!(
            Dispute::<Test>::new(1, *ALICE, BoundedVec::default(), BoundedVec::default()),
            Error::<Test>::DisputeAlreadyExists
        );
    });
}