use cosmwasm_std::{Addr, Binary, Empty, MemoryStorage};
use cw_multi_test::{
    App, BankKeeper, FailingModule, GovFailingModule, IbcFailingModule, MockApiBech32,
    StargateAccepting, WasmKeeper,
};
use neutron_adapters::custom_module::NeutronKeeper;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod neutron_adapters;
pub mod neutron_type_contracts;
pub mod setup;

pub const DENOM_FALLBACK: &str = "ufallback";
pub const DENOM_ATOM: &str = "uatom";
pub const DENOM_NTRN: &str = "untrn";
pub const DENOM_OSMO: &str = "uosmo";
pub const FAUCET: &str = "faucet_addr";
pub const ADMIN: &str = "admin_addr";
pub const ALL_DENOMS: &[&str] = &[DENOM_ATOM, DENOM_NTRN, DENOM_OSMO, DENOM_FALLBACK];
pub const CHAIN_PREFIX: &str = "cosmos";

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
pub const OWNER: &str = "owner";
pub const NOTE: &str = "note";

pub fn make_addr(app: &CustomApp, addr: &str) -> Addr {
    app.api().addr_make(addr)
}
