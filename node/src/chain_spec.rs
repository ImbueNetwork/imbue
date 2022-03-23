use cumulus_primitives_core::ParaId;
use development_runtime::{AccountId, AuraId, Signature, CouncilConfig, TechnicalCommitteeConfig};
use runtime_common::currency::EXISTENTIAL_DEPOSIT;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use std::str::FromStr;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};

use sp_core::{sr25519, Pair, Public};
use sp_runtime::{AccountId32, traits::{IdentifyAccount, Verify}};


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

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

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
		Extensions { relay_chain: "westend".into(), para_id: id.into() },

	)
}

pub fn development_local_config(id: ParaId, environment: &str) -> DevelopmentChainSpec {
	DevelopmentChainSpec::from_genesis(
		// Name
		format!("imbue {} testnet", environment).as_str(),
		// ID
		format!("imbue-{}-testnet", environment).as_str(),
		ChainType::Local,
		move || {
			development_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), get_collator_keys_from_seed("Alice")),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), get_collator_keys_from_seed("Bob")),

				],
				endowed_accounts_local(),
				id.into(),
			)
		},
		Vec::new(),
		None,
		Some("imbue"),
		None,
		Some(imbue_properties()),
		None
	)
}

pub fn development_environment_config(id: ParaId,environment: &str) -> DevelopmentChainSpec {
	DevelopmentChainSpec::from_genesis(
		format!("imbue {} testnet", environment).as_str(),
		// ID
		format!("imbue-{}-testnet", environment).as_str(),
		ChainType::Live,
		move || {
			development_genesis(
				AccountId32::from_str("5F4pGsCKn3AM8CXqiVzpZepZkMBFbiM4qdgCMcg2Pj3yjCNM").unwrap(),
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), get_collator_keys_from_seed("Alice")),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), get_collator_keys_from_seed("Bob")),

				],
				endowed_accounts(),
				id.into(),
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
		None
	)
}

fn endowed_accounts() -> Vec<AccountId> {
	vec![
		AccountId32::from_str("5F4pGsCKn3AM8CXqiVzpZepZkMBFbiM4qdgCMcg2Pj3yjCNM").unwrap(),
	]
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

pub fn development_config() -> DevelopmentChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	DevelopmentChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				AccountId32::from_str("5F4pGsCKn3AM8CXqiVzpZepZkMBFbiM4qdgCMcg2Pj3yjCNM").unwrap(),
				// initial collators.
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
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				1000.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		None,
		None
	)
}




fn development_genesis(
	root_key: AccountId,
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId
) -> development_runtime::GenesisConfig {
	development_runtime::GenesisConfig {
		system: development_runtime::SystemConfig {
			code: development_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: development_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 10 << 60))
				.collect(),
		},
		sudo: development_runtime::SudoConfig { key: Some(root_key) },
		// scheduler: development_runtime::SchedulerConfig {},
		vesting: Default::default(),
		parachain_info: development_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: development_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: development_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		
		aura: development_runtime::AuraConfig {
			authorities:  Default::default(),
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
		polkadot_xcm: development_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
	}
}

pub fn template_session_keys(keys: AuraId) -> development_runtime::SessionKeys {
	development_runtime::SessionKeys { aura: keys }
}

fn testnet_genesis(
	root_key: AccountId,
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> development_runtime::GenesisConfig {
	development_runtime::GenesisConfig {
		system: development_runtime::SystemConfig {
			code: development_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: development_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: development_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: development_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: development_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		council: CouncilConfig {
			phantom: Default::default(),
			members: vec![], // TODO : Set members
		},
		technical_committee: TechnicalCommitteeConfig {
			phantom: Default::default(),
			members: vec![], // TODO : Set members
		},
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: development_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		treasury: Default::default(),
		sudo: development_runtime::SudoConfig { key: Some(root_key) },
		vesting: Default::default(),
	}
}