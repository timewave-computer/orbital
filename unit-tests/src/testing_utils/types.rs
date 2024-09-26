use cosmwasm_schema::cw_serde;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

use cosmwasm_std::{Empty, MemoryStorage};
use cw_multi_test::{
    App, BankKeeper, FailingModule, GovFailingModule, IbcFailingModule, MockApiBech32, WasmKeeper,
};

use super::neutron_adapters::{neutron_module::NeutronKeeper, stargate_module::StargateModule};

#[cw_serde]
pub struct OpenAckVersion {
    pub version: String,
    pub controller_connection_id: String,
    pub host_connection_id: String,
    pub address: String,
    pub encoding: String,
    pub tx_type: String,
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
    StargateModule,
>;
