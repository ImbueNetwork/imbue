// JUST A MOCK TO MAKE BUILD, THESE NEED REGENERATING.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;
use crate::WeightInfoT;


pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> crate::WeightInfoT for WeightInfo<T> {

    fn vote_on_dispute() -> Weight {
        Weight::from_parts(382_002_000, 0)
        .saturating_add(Weight::from_parts(0, 3593))
        .saturating_add(T::DbWeight::get().reads(2))
        .saturating_add(T::DbWeight::get().writes(4))
        
    }
    fn extend_dispute() -> Weight {
        Weight::from_parts(382_002_000, 0)
        .saturating_add(Weight::from_parts(0, 3593))
        .saturating_add(T::DbWeight::get().reads(2))
        .saturating_add(T::DbWeight::get().writes(4))
        
    }
    fn raise_dispute() -> Weight {
        Weight::from_parts(382_002_000, 0)
        .saturating_add(Weight::from_parts(0, 3593))
        .saturating_add(T::DbWeight::get().reads(2))
        .saturating_add(T::DbWeight::get().writes(4))
        
    }
    fn force_succeed_dispute() -> Weight {
        Weight::from_parts(382_002_000, 0)
        .saturating_add(Weight::from_parts(0, 3593))
        .saturating_add(T::DbWeight::get().reads(2))
        .saturating_add(T::DbWeight::get().writes(4))
        
    }
    fn force_fail_dispute() -> Weight {
        Weight::from_parts(382_002_000, 0)
        .saturating_add(Weight::from_parts(0, 3593))
        .saturating_add(T::DbWeight::get().reads(2))
        .saturating_add(T::DbWeight::get().writes(4))
        
    }
    fn calculate_winner() -> Weight {
        Weight::from_parts(382_002_000, 0)
        .saturating_add(Weight::from_parts(0, 3593))
        .saturating_add(T::DbWeight::get().reads(2))
        .saturating_add(T::DbWeight::get().writes(4))
        
    }
}
