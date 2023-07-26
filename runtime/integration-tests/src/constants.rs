use grandpa::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
pub use imbue_kusama_runtime::{AccountId, AuraId, Balance, BlockNumber};
use polkadot_primitives::{AssignmentId, ValidatorId};
pub use polkadot_runtime_parachains::configuration::HostConfiguration;
use polkadot_service::chain_spec::get_authority_keys_from_seed_no_beefy;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, storage::Storage, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    BuildStorage, MultiSignature, Perbill,
};
pub use xcm;
use core::default::Default;
use frame_support::{parameter_types, sp_io, sp_tracing};

pub const XCM_V2: u32 = 3;
pub const XCM_V3: u32 = 2;
pub const REF_TIME_THRESHOLD: u64 = 33;
pub const PROOF_SIZE_THRESHOLD: u64 = 33;

type AccountPublic = <MultiSignature as Verify>::Signer;

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed.
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
    where
        AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub mod accounts {
    use super::*;
    pub const ALICE: &str = "Alice";
    pub const BOB: &str = "Bob";
    pub const CHARLIE: &str = "Charlie";
    pub const DAVE: &str = "Dave";
    pub const EVE: &str = "Eve";
    pub const FERDIE: &str = "Ferdei";
    pub const ALICE_STASH: &str = "Alice//stash";
    pub const BOB_STASH: &str = "Bob//stash";
    pub const CHARLIE_STASH: &str = "Charlie//stash";
    pub const DAVE_STASH: &str = "Dave//stash";
    pub const EVE_STASH: &str = "Eve//stash";
    pub const FERDIE_STASH: &str = "Ferdie//stash";

    pub fn init_balances() -> Vec<AccountId> {
        vec![
            get_account_id_from_seed::<sr25519::Public>(ALICE),
            get_account_id_from_seed::<sr25519::Public>(BOB),
            get_account_id_from_seed::<sr25519::Public>(CHARLIE),
            get_account_id_from_seed::<sr25519::Public>(DAVE),
            get_account_id_from_seed::<sr25519::Public>(EVE),
            get_account_id_from_seed::<sr25519::Public>(FERDIE),
            get_account_id_from_seed::<sr25519::Public>(ALICE_STASH),
            get_account_id_from_seed::<sr25519::Public>(BOB_STASH),
            get_account_id_from_seed::<sr25519::Public>(CHARLIE_STASH),
            get_account_id_from_seed::<sr25519::Public>(DAVE_STASH),
            get_account_id_from_seed::<sr25519::Public>(EVE_STASH),
            get_account_id_from_seed::<sr25519::Public>(FERDIE_STASH),
        ]
    }
}

pub mod collators {
    use super::*;

    pub fn invulnerables_statemint() -> Vec<(AccountId, AuraId)> {
        vec![
            (
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_from_seed::<AuraId>("Alice"),
            ),
            (
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_from_seed::<AuraId>("Bob"),
            ),
        ]
    }

    pub fn invulnerables() -> Vec<(AccountId, AuraId)> {
        vec![
            (
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_from_seed::<AuraId>("Alice"),
            ),
            (get_account_id_from_seed::<sr25519::Public>("Bob"), get_from_seed::<AuraId>("Bob")),
        ]
    }
}

pub mod validators {
    use super::*;

    pub fn initial_authorities() -> Vec<(
        AccountId,
        AccountId,
        BabeId,
        GrandpaId,
        ImOnlineId,
        ValidatorId,
        AssignmentId,
        AuthorityDiscoveryId,
    )> {
        vec![get_authority_keys_from_seed_no_beefy("Alice")]
    }
}

/// The default XCM version to set in genesis config.
pub const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

// Kusama
pub mod kusama {
    use super::*;
    pub const ED: Balance = imbue_kusama_runtime::currency::EXISTENTIAL_DEPOSIT;
    use imbue_kusama_runtime::currency::IMBU as KSM;
    const ENDOWMENT: u128 = 1_000_000 * KSM;
    const STASH: u128 = 100 * KSM;

    pub fn get_host_config() -> HostConfiguration<BlockNumber> {
        HostConfiguration {
            max_upward_queue_count: 10,
            max_upward_queue_size: 51200,
            max_upward_message_size: 51200,
            max_upward_message_num_per_candidate: 10,
            max_downward_message_size: 51200,
            ..Default::default()
        }
    }

    fn session_keys(
        babe: BabeId,
        grandpa: GrandpaId,
        im_online: ImOnlineId,
        para_validator: ValidatorId,
        para_assignment: AssignmentId,
        authority_discovery: AuthorityDiscoveryId,
    ) -> kusama_runtime::SessionKeys {
        kusama_runtime::SessionKeys {
            babe,
            grandpa,
            im_online,
            para_validator,
            para_assignment,
            authority_discovery,
        }
    }

    pub fn genesis() -> Storage {
        let genesis_config = kusama_runtime::GenesisConfig {
            system: kusama_runtime::SystemConfig {
                code: kusama_runtime::WASM_BINARY.unwrap().to_vec(),
            },
            balances: kusama_runtime::BalancesConfig {
                balances: accounts::init_balances()
                    .iter()
                    .map(|k: &AccountId| (k.clone(), ENDOWMENT))
                    .collect(),
            },
            session: kusama_runtime::SessionConfig {
                keys: validators::initial_authorities()
                    .iter()
                    .map(|x| {
                        (
                            x.0.clone(),
                            x.0.clone(),
                            kusama::session_keys(
                                x.2.clone(),
                                x.3.clone(),
                                x.4.clone(),
                                x.5.clone(),
                                x.6.clone(),
                                x.7.clone(),
                            ),
                        )
                    })
                    .collect::<Vec<_>>(),
            },
            staking: kusama_runtime::StakingConfig {
                validator_count: validators::initial_authorities().len() as u32,
                minimum_validator_count: 1,
                stakers: validators::initial_authorities()
                    .iter()
                    .map(|x| {
                        (x.0.clone(), x.1.clone(), STASH, kusama_runtime::StakerStatus::Validator)
                    })
                    .collect(),
                invulnerables: validators::initial_authorities()
                    .iter()
                    .map(|x| x.0.clone())
                    .collect(),
                slash_reward_fraction: Perbill::from_percent(10),
                ..Default::default()
            },
            babe: kusama_runtime::BabeConfig {
                authorities: Default::default(),
                epoch_config: Some(kusama_runtime::BABE_GENESIS_EPOCH_CONFIG),
            },
            configuration: kusama_runtime::ConfigurationConfig { config: get_host_config() },
            ..Default::default()
        };

        genesis_config.build_storage().unwrap()
    }
}

// Imbue
pub mod imbue {
    use common_types::CurrencyId;
    use super::*;
    pub const PARA_ID: u32 = 2000;
    pub const ED: Balance = imbue_kusama_runtime::currency::EXISTENTIAL_DEPOSIT;

    pub fn genesis(para_id: u32) -> Storage {
        let genesis_config = imbue_kusama_runtime::GenesisConfig {
            system: imbue_kusama_runtime::SystemConfig {
                code: imbue_kusama_runtime::WASM_BINARY
                    .expect("WASM binary was not build, please build it!")
                    .to_vec(),
            },
            balances: imbue_kusama_runtime::BalancesConfig {
                balances: accounts::init_balances()
                    .iter()
                    .cloned()
                    .map(|k| (k,  ED.saturating_add(10_000_000_000_000_000)))
                    .collect(),
            },
            orml_tokens: orml_tokens::GenesisConfig {
                balances: accounts::init_balances()
                    .iter()
                    .cloned()
                    .map(|k| (k, CurrencyId::Native,  ED.saturating_add(10_000_000_000_000_000)))
                    .collect(),
            },
            parachain_info: imbue_kusama_runtime::ParachainInfoConfig { parachain_id: para_id.into() },
            collator_selection: imbue_kusama_runtime::CollatorSelectionConfig {
                invulnerables: collators::invulnerables()
                    .iter()
                    .cloned()
                    .map(|(acc, _)| acc)
                    .collect(),
                candidacy_bond: 1 * ED,
                ..Default::default()
            },
            session: imbue_kusama_runtime::SessionConfig {
                keys: collators::invulnerables()
                    .into_iter()
                    .map(|(acc, aura)| {
                        (
                            acc.clone(),                          // account id
                            acc,                                  // validator id
                            imbue_kusama_runtime::SessionKeys { aura }, // session keys
                        )
                    })
                    .collect(),
            },
            aura: Default::default(),
            aura_ext: Default::default(),
            council: Default::default(),
            council_membership: Default::default(),
            democracy: Default::default(),
            treasury: Default::default(),
            technical_committee: Default::default(),
            parachain_system: Default::default(),
            polkadot_xcm: imbue_kusama_runtime::PolkadotXcmConfig{
                safe_xcm_version: Some(SAFE_XCM_VERSION),
            },
            sudo: imbue_kusama_runtime::SudoConfig {
                key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
            },
            ..Default::default()
        };

        genesis_config.build_storage().unwrap()
    }
}