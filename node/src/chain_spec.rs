use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use imbue_kusama_runtime::currency::IMBU;
use imbue_kusama_runtime::{
    AccountId, AuraId, CouncilConfig, CouncilMembershipConfig, DemocracyConfig, Signature,
    TechnicalCommitteeConfig, TechnicalMembershipConfig,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    AccountId32,
};
use std::convert::TryInto;
use std::str::FromStr;

/// Properties for imbue.
pub fn imbue_properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("ss58Format".into(), 42.into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("tokenSymbol".into(), "IMBU".into());
    properties
}

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ImbueKusamaChainSpec = sc_service::GenericChainSpec<imbue_kusama_runtime::GenesisConfig>;

const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
    get_public_from_seed::<AuraId>(seed)
}

pub fn development_local_config(environment: &str) -> ImbueKusamaChainSpec {
    ImbueKusamaChainSpec::from_genesis(
        // Name
        format!("imbue {environment} testnet").as_str(),
        // ID
        format!("imbue-{environment}-testnet").as_str(),
        ChainType::Local,
        move || {
            development_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![(
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_collator_keys_from_seed("Alice"),
                )],
                endowed_accounts_local(),
                Some(250_000_000 * IMBU),
                council_members(),
                tech_committee_members(),
                1000.into(),
            )
        },
        Vec::new(),
        None,
        Some("imbue"),
        None,
        Some(imbue_properties()),
        None,
    )
}

pub fn development_environment_config(environment: &str) -> ImbueKusamaChainSpec {
    ImbueKusamaChainSpec::from_genesis(
        format!("imbue {environment} testnet").as_str(),
        // ID
        format!("imbue-{environment}-testnet").as_str(),
        ChainType::Live,
        move || {
            development_genesis(
                AccountId32::from_str("5DZpUh1ztshcL1Tx6nJrcn9Bnc1RkHc8GehP4eWdspMMqCyi").unwrap(),
                vec![
                    (
                        hex!["17c93b50295e42ba30018fc8ec9e2793faff94b657541da184cc875d66f38cf0"]
                            .into(),
                        hex!["a8465ed76ebfd2ab2fd95e949efdb41bd0208df470fd195b2023a84de500b31b"]
                            .unchecked_into(),
                    ),
                    (
                        hex!["a6ec01606dfd7f0162cc37ecab22c85bd3dd2faa4f8827874c1f86078c6bf403"]
                            .into(),
                        hex!["72992197d9d63d698428f1d94354e8ac6aba3a451d666c6bb433e046499a3665"]
                            .unchecked_into(),
                    ),
                ],
                endowed_accounts(),
                Some(200_000_000 * IMBU),
                council_members(),
                tech_committee_members(),
                1000.into(),
            )
        },
        Vec::new(),
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        Some("imbue"),
        None,
        Some(imbue_properties()),
        None,
    )
}

pub fn imbue_kusama_config() -> ImbueKusamaChainSpec {
    ImbueKusamaChainSpec::from_json_bytes(
        &include_bytes!("../../res/genesis/imbue-kusama-raw.json")[..],
    )
    .unwrap()
}

fn endowed_accounts() -> Vec<AccountId> {
    vec![AccountId32::from_str("5DZpUh1ztshcL1Tx6nJrcn9Bnc1RkHc8GehP4eWdspMMqCyi").unwrap()]
}

fn council_members() -> Vec<AccountId> {
    vec![
        AccountId32::from_str("5F28xL42VWThNonDft4TAQ6rw6a82E2jMsQXS5uMyKiA4ccv").unwrap(),
        AccountId32::from_str("5DZpUh1ztshcL1Tx6nJrcn9Bnc1RkHc8GehP4eWdspMMqCyi").unwrap(),
        AccountId32::from_str("5FsLoiGenakVKDwE7YHe58KLrENj2QZ6zxLLbeUCWKVagMAQ").unwrap(),
        AccountId32::from_str("5EexofvmRpHVFYFehejL7yF3LW1RGZmNR9wAx5fcYXgRUnYp").unwrap(),
        AccountId32::from_str("5FZ991YDZYhsiCmqfsenJLieVFKAgS5rgxTaDbeSu7yP96Bs").unwrap(),
    ]
}

fn tech_committee_members() -> Vec<AccountId> {
    vec![
        AccountId32::from_str("5F28xL42VWThNonDft4TAQ6rw6a82E2jMsQXS5uMyKiA4ccv").unwrap(),
        AccountId32::from_str("5DZpUh1ztshcL1Tx6nJrcn9Bnc1RkHc8GehP4eWdspMMqCyi").unwrap(),
        AccountId32::from_str("5FsLoiGenakVKDwE7YHe58KLrENj2QZ6zxLLbeUCWKVagMAQ").unwrap(),
        AccountId32::from_str("5EexofvmRpHVFYFehejL7yF3LW1RGZmNR9wAx5fcYXgRUnYp").unwrap(),
        AccountId32::from_str("5FZ991YDZYhsiCmqfsenJLieVFKAgS5rgxTaDbeSu7yP96Bs").unwrap(),
    ]
}

fn endowed_accounts_local() -> Vec<AccountId> {
    vec![
        AccountId32::from_str("5DZpUh1ztshcL1Tx6nJrcn9Bnc1RkHc8GehP4eWdspMMqCyi").unwrap(),
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
    ]
}

pub fn get_dev_session_keys(
    keys: imbue_kusama_runtime::AuraId,
) -> imbue_kusama_runtime::SessionKeys {
    imbue_kusama_runtime::SessionKeys { aura: keys }
}

fn development_genesis(
    root_key: AccountId,
    initial_authorities: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    total_issuance: Option<imbue_kusama_runtime::Balance>,
    council_membership: Vec<AccountId>,
    technical_committee_membership: Vec<AccountId>,
    id: ParaId,
) -> imbue_kusama_runtime::GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();

    let (balances, token_balances) = match total_issuance {
        Some(total_issuance) => {
            let balance_per_endowed = total_issuance
                .checked_div(num_endowed_accounts as imbue_kusama_runtime::Balance)
                .unwrap_or(0 as imbue_kusama_runtime::Balance);
            (
                endowed_accounts
                    .iter()
                    .cloned()
                    .map(|k| (k, balance_per_endowed))
                    .collect(),
                endowed_accounts
                    .iter()
                    .cloned()
                    .map(|k| (k, common_runtime::CurrencyId::Native, balance_per_endowed))
                    .collect(),
            )
        }
        None => (vec![], vec![]),
    };

    imbue_kusama_runtime::GenesisConfig {
        system: imbue_kusama_runtime::SystemConfig {
            code: imbue_kusama_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: imbue_kusama_runtime::BalancesConfig { balances },
        orml_asset_registry: Default::default(),
        orml_tokens: imbue_kusama_runtime::OrmlTokensConfig {
            balances: token_balances,
        },
        collator_selection: imbue_kusama_runtime::CollatorSelectionConfig {
            invulnerables: initial_authorities
                .iter()
                .cloned()
                .map(|(acc, _)| acc)
                .collect(),
            candidacy_bond: 1 * IMBU,
            ..Default::default()
        },
        council_membership: CouncilMembershipConfig {
            members: council_membership.try_into().expect("convert error!"),
            phantom: Default::default(),
        },
        technical_membership: TechnicalMembershipConfig {
            members: technical_committee_membership
                .try_into()
                .expect("convert error!"),
            phantom: Default::default(),
        },
        session: imbue_kusama_runtime::SessionConfig {
            keys: initial_authorities
                .iter()
                .cloned()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                // account id
                        acc,                        // validator id
                        get_dev_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        // scheduler: imbue_kusama_runtime::SchedulerConfig {},
        vesting: Default::default(),
        parachain_info: imbue_kusama_runtime::ParachainInfoConfig { parachain_id: id },
        aura: imbue_kusama_runtime::AuraConfig {
            authorities: Default::default(),
        },
        council: CouncilConfig {
            phantom: Default::default(),
            members: vec![], // TODO : Set members
        },
        technical_committee: TechnicalCommitteeConfig {
            phantom: Default::default(),
            members: vec![], // TODO : Set members
        },
        democracy: DemocracyConfig::default(),
        treasury: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: imbue_kusama_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
        },
    }
}
