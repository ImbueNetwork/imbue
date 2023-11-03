use common_runtime::Balance;
use common_types::currency_decimals;
use sp_runtime::BuildStorage;
pub use imbue_kusama_runtime::{AccountId, CurrencyId, Runtime, RuntimeOrigin, System};
/// Parachain Ids
pub const PARA_ID_DEVELOPMENT: u32 = 2121;
pub const PARA_ID_SIBLING: u32 = 2110;
// pub const PARA_ID_KARURA: u32 = 2000;

pub struct ExtBuilder {
    balances: Vec<(AccountId, CurrencyId, Balance)>,
    parachain_id: u32,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![],
            parachain_id: PARA_ID_DEVELOPMENT,
        }
    }
}

impl ExtBuilder {
    pub fn balances(mut self, balances: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    pub fn parachain_id(mut self, parachain_id: u32) -> Self {
        self.parachain_id = parachain_id;
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
        let native_currency_id = imbue_kusama_runtime::CurrencyId::Native;
        pallet_balances::GenesisConfig::<Runtime> {
            balances: self
                .balances
                .clone()
                .into_iter()
                .filter(|(_, currency_id, _)| *currency_id == native_currency_id)
                .map(|(account_id, _, initial_balance)| (account_id, initial_balance))
                .collect::<Vec<_>>(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        orml_tokens::GenesisConfig::<Runtime> {
            balances: self
                .balances
                .into_iter()
                .filter(|(_, currency_id, _)| *currency_id != native_currency_id)
                .collect::<Vec<_>>(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        parachain_info::GenesisConfig::<Runtime> {
            parachain_id: self.parachain_id.into(),
            ..Default::default()
        }.assimilate_storage(&mut t)
        .unwrap();

        pallet_xcm::GenesisConfig::<Runtime> {
            safe_xcm_version: Some(3),
            ..Default::default()
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub fn native_amount(amount: Balance) -> Balance {
    amount * dollar(currency_decimals::NATIVE)
}

pub fn mgx_amount(amount: Balance) -> Balance {
    amount * dollar(currency_decimals::MGX)
}

// pub fn kar_amount(amount: Balance) -> Balance {
//     amount * dollar(currency_decimals::KAR)
// }

pub fn ksm_amount(amount: Balance) -> Balance {
    amount * dollar(currency_decimals::KSM)
}

pub fn dollar(decimals: u32) -> Balance {
    10u128.saturating_pow(decimals)
}
