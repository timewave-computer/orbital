use cosmwasm_std::{coin, coins, Addr, Coin};
use cw_multi_test::{
    BasicAppBuilder, Executor, MockApiBech32, SimpleAddressGenerator, StargateAccepting, WasmKeeper,
};

use super::{
    consts::{ALL_DENOMS, CHAIN_PREFIX, DENOM_NTRN, FAUCET, NOTE, OWNER, USER_1},
    neutron_adapters::{
        custom_module::NeutronKeeper, neutron_type_contracts::orbital_core_contract,
    },
    types::CustomApp,
};

pub trait BaseSuite {
    fn get_app(&self) -> &CustomApp;

    fn query_balance(&self, addr: &Addr, denom: &str) -> Coin {
        let app = self.get_app();
        app.wrap().query_balance(addr, denom).unwrap()
    }

    fn query_all_balances(&self, addr: &Addr) -> Vec<Coin> {
        let app = self.get_app();
        app.wrap().query_all_balances(addr).unwrap()
    }

    fn assert_balance(&self, addr: impl Into<String>, coin: Coin) {
        let app = self.get_app();
        let bal = app.wrap().query_balance(addr, &coin.denom).unwrap();
        assert_eq!(bal, coin);
    }
}

pub struct SuiteBuilder {
    pub faucet: Addr,
    pub admin: Addr,
    pub note: Addr,
    pub app: CustomApp,
    pub orbital_core_code_id: u64,
    pub user_addr: Addr,
}

impl Default for SuiteBuilder {
    fn default() -> Self {
        let mut app = BasicAppBuilder::new_custom()
            .with_custom(NeutronKeeper::new(CHAIN_PREFIX))
            .with_stargate(StargateAccepting)
            .with_api(MockApiBech32::new(CHAIN_PREFIX))
            .with_wasm(WasmKeeper::default().with_address_generator(SimpleAddressGenerator))
            .build(|r, _, s| {
                let balances: Vec<Coin> = ALL_DENOMS
                    .iter()
                    .map(|d| coin(1_000_000_000_000_000_000_000_000_u128, d.to_string()))
                    .collect();

                r.bank
                    .init_balance(
                        s,
                        &MockApiBech32::new(CHAIN_PREFIX).addr_make(FAUCET),
                        balances,
                    )
                    .unwrap();
            });

        let code_id = app.store_code(orbital_core_contract());

        let owner_addr = app.api().addr_make(OWNER);
        let faucet_addr = app.api().addr_make(FAUCET);
        let note_addr = app.api().addr_make(NOTE);
        let user_addr = app.api().addr_make(USER_1);

        app.send_tokens(
            faucet_addr.clone(),
            user_addr.clone(),
            &coins(1_000_000, DENOM_NTRN),
        )
        .unwrap();

        Self {
            faucet: faucet_addr,
            admin: owner_addr,
            note: note_addr,
            app,
            orbital_core_code_id: code_id,
            user_addr,
        }
    }
}

impl SuiteBuilder {
    pub fn build(self) -> CustomApp {
        self.app
    }
}

pub fn make_addr(app: &CustomApp, addr: &str) -> Addr {
    app.api().addr_make(addr)
}
