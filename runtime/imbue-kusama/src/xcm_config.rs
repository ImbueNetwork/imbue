use crate::{AllPalletsWithSystem, Balances, ConstU32};
use frame_system::EnsureRoot;
use sp_runtime::traits::{Convert, Zero};
use sp_std::{marker::PhantomData, prelude::*};

// A few exports that help ease life for downstream crates.
pub use common_runtime::{
    asset_registry::AuthorityOrigin,
    common_xcm::general_key,
    parachains,
    xcm_fees::{default_per_second, ksm_per_second, native_per_second, WeightToFee},
    EnsureRootOr,
};
pub use common_types::{currency_decimals, CurrencyId, CustomMetadata};
pub use frame_support::{
    construct_runtime,
    dispatch::DispatchClass,
    ensure, parameter_types,
    traits::{
        fungibles, Contains, Currency as PalletCurrency, EnsureOriginWithArg, EqualPrivilegeOnly,
        Everything, Get, Imbalance, IsInVec, Nothing, OnUnbalanced, Randomness,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
        ConstantMultiplier, IdentityFee, Weight,
    },
    PalletId, StorageValue,
};
use orml_asset_registry::{AssetRegistryTrader, FixedRateAssetRegistryTrader};
use orml_traits::{
    location::AbsoluteReserveProvider, parameter_type_with_key, FixedConversionRateProvider,
};
use orml_xcm_support::{IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use xcm::v3::Junction::GeneralKey;
use xcm::{v3::prelude::*, v3::Weight as XcmWeight};

use pallet_xcm::XcmPassthrough;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
    ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
    SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
};
use xcm_executor::XcmExecutor;

use pallet_collective::EnsureProportionAtLeast;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::DispatchError;

parameter_types! {
    // One XCM operation is 100_000_000 weight - almost certainly a conservative estimate.
    pub UnitWeightCost: XcmWeight = XcmWeight::from_parts(1_000_000_000, 1024);
    pub const MaxInstructions: u32 = 100;

}

use super::{
    AccountId, Balance, CouncilCollective, Currencies, OrmlAssetRegistry, ParachainInfo,
    ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, UnknownTokens,
    XcmpQueue,
};

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the default `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
    // Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
    // transaction from the Root origin.
    ParentAsSuperuser<RuntimeOrigin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<RuntimeOrigin>,
);

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    // Expected responses are OK.
    AllowKnownQueryResponses<PolkadotXcm>,
    // Subscriptions for version tracking are OK.
    AllowSubscriptionsFrom<Everything>,
);

parameter_types! {
    pub const KsmLocation: MultiLocation = MultiLocation::parent();
    pub const MaxAssetsIntoHolding: u32 = 64;
    pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
    pub const RelayNetwork: NetworkId = NetworkId::Kusama;
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
    // A `MultiLocation` that can be reached via `XcmRouter`. Used only in benchmarks.
    // If `None`, the benchmarks that depend on a reachable destination will be skipped.
    pub ReachableDest: Option<MultiLocation> = None;
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type RuntimeCall = RuntimeCall;
    type AssetClaims = PolkadotXcm;
    type AssetTransactor = LocalAssetTransactor;
    type AssetTrap = PolkadotXcm;
    type Barrier = Barrier;
    type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
    type IsTeleporter = ();
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type ResponseHandler = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
    type Trader = Trader;
    type UniversalLocation = UniversalLocation;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type XcmSender = XcmRouter;
    type AssetLocker = ();
    type AssetExchanger = ();
    type FeeManager = ();
    type MessageExporter = ();
    type UniversalAliases = ();
    type PalletInstancesInfo = AllPalletsWithSystem;
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
}

pub type LocalAssetTransactor = MultiCurrencyAdapter<
    Currencies,
    UnknownTokens,
    IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
    AccountId,
    LocationToAccountId,
    CurrencyId,
    CurrencyIdConvert,
    DepositFailureHandler,
    // DepositToAlternative<TreasuryAccount, Currencies, AssetId, AccountId, Balance>,
>;

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
    fn take_revenue(revenue: MultiAsset) {
        if let MultiAsset {
            id: Concrete(_location),
            fun: Fungible(_amount),
        } = revenue
        {
            // TODO(sam): implement this
        }
    }
}

parameter_types! {
    pub KsmPerSecond: (AssetId, u128, u128) = (MultiLocation::parent().into(), ksm_per_second(), ksm_per_second());

    pub CanonicalImbuePerSecond: (AssetId, u128, u128) = (
        MultiLocation::new(
            0,
            X1(general_key(parachains::kusama::imbue::IMBU_KEY)),
        ).into(),
        native_per_second(),
        native_per_second(),
    );

    pub ImbuPerSecond: (AssetId, u128, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kusama::imbue::ID), general_key(parachains::kusama::imbue::IMBU_KEY))
        ).into(),
        native_per_second(),
        native_per_second(),
    );

    pub MgxPerSecond: (AssetId, u128, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kusama::mangata::ID), general_key(parachains::kusama::mangata::MGX_KEY))
        ).into(),
        ksm_per_second() * 50,
        ksm_per_second() * 50
    );

    pub AUsdPerSecond: (AssetId, u128, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kusama::karura::ID), general_key(parachains::kusama::karura::AUSD_KEY))
        ).into(),
        ksm_per_second() * 50,
        ksm_per_second() * 50
    );

    pub KarPerSecond: (AssetId, u128, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kusama::karura::ID), general_key(parachains::kusama::karura::KAR_KEY))
        ).into(),
        ksm_per_second() * 100,
        ksm_per_second() * 100,
    );
}

pub type Trader = (
    FixedRateOfFungible<CanonicalImbuePerSecond, ToTreasury>,
    FixedRateOfFungible<ImbuPerSecond, ToTreasury>,
    FixedRateOfFungible<KsmPerSecond, ToTreasury>,
    AssetRegistryTrader<FixedRateAssetRegistryTrader<FeePerSecondProvider>, ToTreasury>,
    FixedRateOfFungible<AUsdPerSecond, ToTreasury>,
    FixedRateOfFungible<KarPerSecond, ToTreasury>,
    FixedRateOfFungible<MgxPerSecond, ToTreasury>,
);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Nothing;
    // ^ Disable dispatchable execute on the XCM pallet.
    // Needs to be `Everything` for local testing.
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Everything;
    type XcmReserveTransferFilter = Nothing;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type TrustedLockers = ();
    type UniversalLocation = UniversalLocation;
    type SovereignAccountOf = LocationToAccountId;
    type MaxLockers = ConstU32<8>;
    type WeightInfo = pallet_xcm::TestWeightInfo;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    // ^ Override for AdvertisedXcmVersion default
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    #[cfg(feature = "runtime-benchmarks")]
    type ReachableDest = ReachableDest;
    type AdminOrigin = EnsureRoot<AccountId>;
    type MaxRemoteLockConsumers = ConstU32<0>;
    type RemoteLockConsumerIdentifier = ();

}

impl orml_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SovereignOrigin = MoreThanHalfCouncil;
}

pub struct DepositFailureHandler;

impl<CurrencyId, AccountId, Balance> orml_xcm_support::OnDepositFail<CurrencyId, AccountId, Balance>
    for DepositFailureHandler
{
    fn on_deposit_currency_fail(
        err: DispatchError,
        _currency_id: CurrencyId,
        _who: &AccountId,
        _amount: Balance,
    ) -> xcm::latest::Result {
        Err(XcmError::FailedToTransactAsset(err.into()))
    }
}

/// CurrencyIdConvert
/// This type implements conversions from our `CurrencyId` type into `MultiLocation` and vice-versa.
/// A currency locally is identified with a `CurrencyId` variant but in the network it is identified
/// in the form of a `MultiLocation`, in this case a pair (Para-Id, Currency-Id).
pub struct CurrencyIdConvert;

/// Convert an incoming `MultiLocation` into a `CurrencyId` if possible.
/// Here we need to know the canonical representation of all the tokens we handle in order to
/// correctly convert their `MultiLocation` representation into our internal `CurrencyId` type.
impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<CurrencyId> {
        if location == MultiLocation::parent() {
            return Some(CurrencyId::KSM);
        }

        match location.clone() {
            MultiLocation {
                parents: 0,
                interior: X1(GeneralKey { data, length }),
            } => match &data[..(length as usize)] {
                parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                _ => OrmlAssetRegistry::location_to_asset_id(location),
            },
            MultiLocation {
                parents: 1,
                interior: X2(Parachain(para_id), GeneralKey { data, length }),
            } => match para_id {
                parachains::kusama::karura::ID => match &data[..(length as usize)] {
                    parachains::kusama::karura::AUSD_KEY => Some(CurrencyId::AUSD),
                    parachains::kusama::karura::KAR_KEY => Some(CurrencyId::KAR),
                    parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                    _ => OrmlAssetRegistry::location_to_asset_id(location),
                },
                parachains::kusama::mangata::ID => match &data[..(length as usize)] {
                    parachains::kusama::mangata::MGX_KEY => Some(CurrencyId::MGX),
                    parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                    _ => OrmlAssetRegistry::location_to_asset_id(location),
                },
                parachains::kusama::imbue::ID => match &data[..(length as usize)] {
                    parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                    _ => OrmlAssetRegistry::location_to_asset_id(location),
                },

                id if id == u32::from(ParachainInfo::get()) => match &data[..(length as usize)] {
                    parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                    _ => OrmlAssetRegistry::location_to_asset_id(location),
                },
                _ => OrmlAssetRegistry::location_to_asset_id(location),
            },
            _ => OrmlAssetRegistry::location_to_asset_id(location),
        }
    }
}

impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(asset: MultiAsset) -> Option<CurrencyId> {
        if let MultiAsset {
            id: Concrete(location),
            ..
        } = asset
        {
            Self::convert(location)
        } else {
            None
        }
    }
}

/// Convert our `CurrencyId` type into its `MultiLocation` representation.
/// Other chains need to know how this conversion takes place in order to
/// handle it on their side.
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: CurrencyId) -> Option<MultiLocation> {
        match id {
            CurrencyId::KSM => Some(MultiLocation::parent()),
            CurrencyId::AUSD => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(parachains::kusama::karura::ID),
                    general_key(parachains::kusama::karura::AUSD_KEY),
                ),
            )),
            CurrencyId::KAR => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(parachains::kusama::karura::ID),
                    general_key(parachains::kusama::karura::KAR_KEY),
                ),
            )),
            CurrencyId::MGX => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(parachains::kusama::mangata::ID),
                    general_key(parachains::kusama::mangata::MGX_KEY),
                ),
            )),
            CurrencyId::Native => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(ParachainInfo::get().into()),
                    general_key(parachains::kusama::imbue::IMBU_KEY),
                ),
            )),
            CurrencyId::ForeignAsset(_) => OrmlAssetRegistry::multilocation(&id).ok()?,
        }
    }
}

/// All council members must vote yes to create this origin.
type HalfOfCouncil = EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
/// A majority of the Unit body from Rococo over XCM is our required administration origin.
pub type AdminOrigin = EnsureRootOr<HalfOfCouncil>;
pub type MoreThanHalfCouncil = EnsureRootOr<HalfOfCouncil>;

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

parameter_types! {
    //TODO(Sam): we may need to fine tune this value later on
    pub const BaseXcmWeight: XcmWeight = XcmWeight::from_parts(1_000_000_000, 1024);
    pub const MaxAssetsForTransfer: usize = 2;
}

impl orml_xtokens::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type SelfLocation = SelfLocation;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type MinXcmFee = ParachainMinFee;
    type MultiLocationsFilter = Everything;
    type ReserveProvider = AbsoluteReserveProvider;
    type UniversalLocation = UniversalLocation;
}

parameter_types! {
    pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::get().into())));
}

parameter_type_with_key! {
    pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
        None
    };
}

pub struct FeePerSecondProvider;
impl FixedConversionRateProvider for FeePerSecondProvider {
    fn get_fee_per_second(location: &MultiLocation) -> Option<u128> {
        OrmlAssetRegistry::fetch_metadata_by_location(location)?
            .additional
            .xcm
            .fee_per_second
    }
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account: AccountId) -> MultiLocation {
        X1(AccountId32 {
            network: Some(NetworkId::Kusama),
            id: account.into(),
        })
        .into()
    }
}


