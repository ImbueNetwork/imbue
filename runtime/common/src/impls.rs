//! Some configurable implementations as associated type for the substrate runtime.

use super::*;
use core::marker::PhantomData;
use frame_support::sp_runtime::app_crypto::sp_core::U256;
use frame_support::traits::{Currency, OnUnbalanced};
use frame_support::weights::{
    WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use frame_system::pallet::Config as SystemConfig;
use pallet_authorship::{Config as AuthorshipConfig, Pallet as Authorship};
use pallet_balances::{Config as BalancesConfig, Pallet as Balances};
use smallvec::smallvec;
use sp_arithmetic::Perbill;
use sp_std::vec;
use sp_std::vec::Vec;

pub struct DealWithFees<Config>(PhantomData<Config>);
pub type NegativeImbalance<Config> =
    <Balances<Config> as Currency<<Config as SystemConfig>::AccountId>>::NegativeImbalance;
impl<Config> OnUnbalanced<NegativeImbalance<Config>> for DealWithFees<Config>
where
    Config: AuthorshipConfig + BalancesConfig + SystemConfig,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance<Config>) {
        if let Some(who) = Authorship::<Config>::author() {
            Balances::<Config>::resolve_creating(&who, amount);
        }
    }
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec!(WeightToFeeCoefficient {
            coeff_integer: 315000,
            coeff_frac: Perbill::zero(),
            negative: false,
            degree: 1,
        })
    }
}

impl common_traits::BigEndian<Vec<u8>> for TokenId {
    fn to_big_endian(&self) -> Vec<u8> {
        let mut data = vec![0; 32];
        self.0.to_big_endian(&mut data);
        data
    }
}

impl From<U256> for TokenId {
    fn from(v: U256) -> Self {
        Self(v)
    }
}

impl From<u16> for InstanceId {
    fn from(v: u16) -> Self {
        Self(v as u128)
    }
}

impl From<u128> for InstanceId {
    fn from(v: u128) -> Self {
        Self(v)
    }
}
