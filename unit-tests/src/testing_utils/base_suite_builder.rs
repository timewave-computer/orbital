use cosmwasm_std::{coin, Addr, Coin};
use cw_multi_test::{
    BasicAppBuilder, MockApiBech32, SimpleAddressGenerator, StargateAccepting, WasmKeeper,
};

use super::{
    neutron_adapters::custom_module::NeutronKeeper, neutron_type_contracts::orbital_core_contract,
    CustomApp, ALL_DENOMS, CHAIN_PREFIX, FAUCET, NOTE, OWNER,
};

pub struct SuiteBuilder {
    pub faucet: Addr,
    pub admin: Addr,
    pub note: Addr,
    pub app: CustomApp,
    pub orbital_core_code_id: u64,
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

        Self {
            faucet: faucet_addr,
            admin: owner_addr,
            note: note_addr,
            app,
            orbital_core_code_id: code_id,
        }
    }
}

impl SuiteBuilder {
    pub fn build(self) -> CustomApp {
        self.app
    }
}
