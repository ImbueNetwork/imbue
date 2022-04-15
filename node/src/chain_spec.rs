use cumulus_primitives_core::ParaId;
use development_runtime::{AccountId, AuraId, CouncilConfig, Signature, TechnicalCommitteeConfig};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use development_runtime::currency::IMBU;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    AccountId32,
};

/// Properties for imbue.
pub fn imbue_properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("ss58Format".into(), 31.into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("tokenSymbol".into(), "IMBU".into());
    properties
}

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type DevelopmentChainSpec = sc_service::GenericChainSpec<development_runtime::GenesisConfig>;

/// Specialized `ChainSpec` for the shell parachain runtime.
pub type ShellChainSpec = sc_service::GenericChainSpec<shell_runtime::GenesisConfig, Extensions>;

const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
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
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
    get_public_from_seed::<AuraId>(seed)
}

fn shell_testnet_genesis(parachain_id: ParaId) -> shell_runtime::GenesisConfig {
    shell_runtime::GenesisConfig {
        system: shell_runtime::SystemConfig {
            code: shell_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        parachain_info: shell_runtime::ParachainInfoConfig { parachain_id },
        parachain_system: Default::default(),
    }
}

pub fn get_shell_chain_spec(id: ParaId) -> ShellChainSpec {
    ShellChainSpec::from_genesis(
        "Shell Local Testnet",
        "shell_local_testnet",
        ChainType::Local,
        move || shell_testnet_genesis(id.into()),
        vec![],
        None,
        Some("imbue"),
        None,
        Some(imbue_properties()),
        Extensions {
            relay_chain: "westend".into(),
            para_id: id.into(),
        },
    )
}

pub fn development_local_config(id: ParaId, environment: &str) -> DevelopmentChainSpec {
    DevelopmentChainSpec::from_genesis(
        // Name
        format!("imbue {} testnet", environment).as_str(),
        // ID
        format!("imbue-{}-testnet", environment).as_str(),
        ChainType::Development,
        move || {
            development_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![(
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_collator_keys_from_seed("Alice"),
                )],
                endowed_accounts_local(),
                Some(10_000_000 * IMBU),
                id,
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

pub fn development_environment_config(id: ParaId, environment: &str) -> DevelopmentChainSpec {
    DevelopmentChainSpec::from_genesis(
        format!("imbue {} testnet", environment).as_str(),
        // ID
        format!("imbue-{}-testnet", environment).as_str(),
        ChainType::Live,
        move || {
            development_genesis(
                AccountId32::from_str("5F28xL42VWThNonDft4TAQ6rw6a82E2jMsQXS5uMyKiA4ccv").unwrap(),
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                endowed_accounts(),
                Some(10_000_000 * IMBU),
                id,
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

fn endowed_accounts() -> Vec<AccountId> {
    vec![AccountId32::from_str("5F4pGsCKn3AM8CXqiVzpZepZkMBFbiM4qdgCMcg2Pj3yjCNM").unwrap()]
}

fn endowed_accounts_local() -> Vec<AccountId> {
    vec![
        AccountId32::from_str("5F4pGsCKn3AM8CXqiVzpZepZkMBFbiM4qdgCMcg2Pj3yjCNM").unwrap(),
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        // get_account_id_from_seed::<sr25519::Public>("Charlie"),
        // get_account_id_from_seed::<sr25519::Public>("Dave"),
        // get_account_id_from_seed::<sr25519::Public>("Eve"),
        // get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        // get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        // get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        // get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        // get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
        // get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
        // get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
    ]
}

pub fn get_dev_session_keys(keys: development_runtime::AuraId) -> development_runtime::SessionKeys {
    development_runtime::SessionKeys { aura: keys }
}

fn development_genesis(
    root_key: AccountId,
    initial_authorities: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    total_issuance: Option<development_runtime::Balance>,
    id: ParaId,
) -> development_runtime::GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();

    let (balances, token_balances) = match total_issuance {
        Some(total_issuance) => {
            let balance_per_endowed = total_issuance
                .checked_div(num_endowed_accounts as development_runtime::Balance)
                .unwrap_or(0 as development_runtime::Balance);
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

    development_runtime::GenesisConfig {
        system: development_runtime::SystemConfig {
            code: development_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: development_runtime::BalancesConfig { balances: balances },
        sudo: development_runtime::SudoConfig {
            key: Some(root_key),
        },
        orml_tokens: development_runtime::OrmlTokensConfig {
            balances: token_balances,
        },
        collator_selection: development_runtime::CollatorSelectionConfig {
            invulnerables: initial_authorities
                .iter()
                .cloned()
                .map(|(acc, _)| acc)
                .collect(),
            candidacy_bond: 1 * IMBU,
            ..Default::default()
        },
        session: development_runtime::SessionConfig {
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
        // scheduler: development_runtime::SchedulerConfig {},
        vesting: Default::default(),
        parachain_info: development_runtime::ParachainInfoConfig { parachain_id: id },
        aura: development_runtime::AuraConfig {
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
        treasury: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
    }
}
