use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use development_runtime::{AccountId, AuraId, Signature, CouncilConfig, TechnicalCommitteeConfig};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use std::str::FromStr;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};

use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
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
					get_from_seed::<AuraId>("Alice"),
					get_from_seed::<AuraId>("Bob"),
				],
				endowed_accounts_local(),
				id.into(),
			)
		},
		Vec::new(),
		None,
		Some("imbue"),
		Some(imbue_properties()),
		Default::default()
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
					hex!["7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73"]
					.unchecked_into(),
					hex!["287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d"]
					.unchecked_into()
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
		Some(imbue_properties()),
		Default::default()
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

fn development_genesis(
	root_key: AccountId,
	initial_authorities: Vec<AuraId>,
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
		sudo: development_runtime::SudoConfig { key: root_key },
		scheduler: development_runtime::SchedulerConfig {},
		vesting: Default::default(),
		parachain_info: development_runtime::ParachainInfoConfig { parachain_id: id },
		aura: development_runtime::AuraConfig {
			authorities: initial_authorities,
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
