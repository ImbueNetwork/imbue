use crate::milestone_origin::FundingType;
use sp_std::marker::PhantomData;

pub trait DepositCalculator<StorageType> {
    type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;
    type Balance: AtLeast32BitUnsigned
    + Codec
    + Copy
    + Debug
    + Default
    + MaybeSerializeDeserialize
    + Member
    + Parameter
    + Zero;
    pub fn get_deposit(t: StorageType, currency: Self::CurrencyId) -> Self::Balance;
}
