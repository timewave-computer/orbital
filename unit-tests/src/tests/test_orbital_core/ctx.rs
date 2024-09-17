use cosmwasm_std::{coin, Addr, Binary, Coin, Empty, GrpcQuery, MemoryStorage, StdResult};
use cw_multi_test::{
    error::AnyResult, App, AppResponse, BankKeeper, BasicAppBuilder, Executor, FailingModule,
    GovFailingModule, IbcFailingModule, MockApiBech32, SimpleAddressGenerator, StargateAccepting,
    WasmKeeper,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use orbital_core::{
    account_types::UncheckedOrbitalDomainConfig,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{OrbitalDomainConfig, UserConfig},
};

use crate::testing_utils::{
    neutron_adapters::{custom_keepers::CustomStargateKeeper, custom_module::NeutronKeeper},
    neutron_type_contracts::orbital_core_contract,
};

pub const DENOM_FALLBACK: &str = "ufallback";
pub const DENOM_ATOM: &str = "uatom";
pub const DENOM_NTRN: &str = "untrn";
pub const DENOM_OSMO: &str = "uosmo";
pub const FAUCET: &str = "faucet_addr";
pub const ADMIN: &str = "admin_addr";
pub const ALL_DENOMS: &[&str] = &[DENOM_ATOM, DENOM_NTRN, DENOM_OSMO, DENOM_FALLBACK];
pub const CHAIN_PREFIX: &str = "cosmos";

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

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StargateMsg {
    /// Stargate message type.
    pub type_url: String,
    /// Stargate message body.
    pub value: Binary,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StargateQuery {
    /// Fully qualified service path used for routing, e.g. custom/cosmos_sdk.x.bank.v1.Query/QueryBalance.
    pub path: String,
    /// Expected protobuf message type (not any), binary encoded.
    pub data: Binary,
}

pub type CustomApp = App<
    BankKeeper,
    MockApiBech32,
    MemoryStorage,
    NeutronKeeper,
    WasmKeeper<NeutronMsg, NeutronQuery>,
    FailingModule<Empty, Empty, Empty>,
    FailingModule<Empty, Empty, Empty>,
    IbcFailingModule,
    GovFailingModule,
    StargateAccepting,
>;
const OWNER: &str = "owner";
const NOTE: &str = "note";

pub struct SuiteBuilder {}

pub struct Suite {
    pub app: CustomApp,
    pub owner: Addr,
    pub orbital: Addr,
    pub note: Addr,
}

fn make_addr(app: &CustomApp, addr: &str) -> Addr {
    app.api().addr_make(addr)
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
