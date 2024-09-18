use cosmwasm_std::{coin, Addr, Coin, StdResult};
use cw_multi_test::{
    error::AnyResult, AppResponse, BasicAppBuilder, Executor, MockApiBech32,
    SimpleAddressGenerator, StargateAccepting, WasmKeeper,
};
use orbital_core::{
    account_types::UncheckedOrbitalDomainConfig,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{OrbitalDomainConfig, UserConfig},
};

use crate::testing_utils::{
    make_addr, neutron_adapters::custom_module::NeutronKeeper,
    neutron_type_contracts::orbital_core_contract, CustomApp, ALL_DENOMS, CHAIN_PREFIX, FAUCET,
    NOTE, OWNER,
};

pub struct OrbitalCoreInstantiate {
    pub msg: InstantiateMsg,
}

impl Default for OrbitalCoreInstantiate {
    fn default() -> Self {
        OrbitalCoreInstantiate {
            msg: InstantiateMsg {
                owner: "TODO".to_string(),
            },
        }
    }
}

pub struct SuiteBuilder {}

pub struct Suite {
    pub app: CustomApp,
    pub owner: Addr,
    pub orbital: Addr,
    pub note: Addr,
}

impl Suite {
    pub fn register_user(&mut self, user_addr: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            make_addr(&self.app, user_addr),
            self.orbital.clone(),
            &ExecuteMsg::RegisterUser {},
            &[],
        )
    }
    pub fn query_domain(&mut self, domain: &str) -> StdResult<OrbitalDomainConfig> {
        self.app.wrap().query_wasm_smart(
            self.orbital.clone(),
            &QueryMsg::OrbitalDomain {
                domain: domain.to_string(),
            },
        )
    }
    pub fn query_user(&mut self, user: &str) -> StdResult<UserConfig> {
        self.app.wrap().query_wasm_smart(
            self.orbital.clone(),
            &QueryMsg::UserConfig {
                user: make_addr(&self.app, user).to_string(),
            },
        )
    }
    pub fn register_new_domain(
        &mut self,
        domain: &str,
        account_type: UncheckedOrbitalDomainConfig,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            self.owner.clone(),
            self.orbital.clone(),
            &ExecuteMsg::RegisterNewDomain {
                domain: domain.to_string(),
                account_type,
            },
            &[],
        )
    }
}

impl Default for Suite {
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

                // r.custom
                //     .add_local_channel(s, NTRN_HUB_CHANNEL.0, NTRN_HUB_CHANNEL.1)
                //     .unwrap();
                // r.custom
                //     .add_local_channel(s, NTRN_OSMO_CHANNEL.0, NTRN_OSMO_CHANNEL.1)
                //     .unwrap();
                // r.custom
                //     .add_local_channel(s, NTRN_STRIDE_CHANNEL.0, NTRN_STRIDE_CHANNEL.1)
                //     .unwrap();

                // r.custom
                //     .add_remote_channel(s, HUB_OSMO_CHANNEL.0, HUB_OSMO_CHANNEL.1)
                //     .unwrap();

                // r.custom
                //     .add_remote_channel(s, HUB_STRIDE_CHANNEL.0, HUB_STRIDE_CHANNEL.1)
                //     .unwrap();
            });

        let code_id = app.store_code(orbital_core_contract());

        let owner_addr = app.api().addr_make(OWNER);
        let note_addr = app.api().addr_make(NOTE);

        let addr = app
            .instantiate_contract(
                code_id,
                owner_addr.clone(),
                &InstantiateMsg {
                    owner: owner_addr.to_string(),
                },
                &[],
                "orbital-core",
                None,
            )
            .unwrap();

        Self {
            app,
            owner: owner_addr,
            orbital: addr,
            note: note_addr,
        }
    }
}
