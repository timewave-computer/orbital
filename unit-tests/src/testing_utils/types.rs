use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// use super::setup::base_suite::CustomApp;
use cosmwasm_std::{Empty, MemoryStorage};
use cw_multi_test::{
    App, BankKeeper, FailingModule, GovFailingModule, IbcFailingModule, MockApiBech32, WasmKeeper,
};

use super::neutron_adapters::{custom_module::NeutronKeeper, stargate_module::StargateModule};

// use super::neutron_adapters::custom_keepers::CustomStargateKeeper;

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

#[cw_serde]
pub struct OpenAckVersion {
    pub version: String,
    pub controller_connection_id: String,
    pub host_connection_id: String,
    pub address: String,
    pub encoding: String,
    pub tx_type: String,
}

#[cw_serde]
pub enum AcknowledgementResult {
    /// Success - Got success acknowledgement in sudo with array of message item types in it
    Success(Vec<String>),
    /// Error - Got error acknowledgement in sudo with payload message in it and error details
    Error((String, String)),
    /// Timeout - Got timeout acknowledgement in sudo with payload message in it
    Timeout(String),
}

#[cw_serde]
pub struct SudoPayload {
    pub message: String,
    pub port_id: String,
}

// https://github.com/strangelove-ventures/packet-forward-middleware/blob/main/router/types/forward.go
#[cw_serde]
pub struct PacketMetadata {
    pub forward: Option<ForwardMetadata>,
}

#[cw_serde]
pub struct ForwardMetadata {
    pub receiver: String,
    pub port: String,
    pub channel: String,
}

// #[derive(Clone, PartialEq, ::prost::Message)]
// pub struct MsgTransfer {
//     /// the port on which the packet will be sent
//     #[prost(string, tag = "1")]
//     pub source_port: String,
//     /// the channel by which the packet will be sent
//     #[prost(string, tag = "2")]
//     pub source_channel: String,
//     /// the tokens to be transferred
//     #[prost(message, optional, tag = "3")]
//     pub token: Option<cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
//     /// the sender address
//     #[prost(string, tag = "4")]
//     pub sender: String,
//     /// the recipient address on the destination chain
//     #[prost(string, tag = "5")]
//     pub receiver: String,
//     /// Timeout height relative to the current block height.
//     /// The timeout is disabled when set to 0.
//     #[prost(message, optional, tag = "6")]
//     pub timeout_height: Option<IbcCounterpartyHeight>,
//     /// Timeout timestamp in absolute nanoseconds since unix epoch.
//     /// The timeout is disabled when set to 0.
//     #[prost(uint64, tag = "7")]
//     pub timeout_timestamp: u64,
//     #[prost(string, tag = "8")]
//     pub memo: String,
// }

// #[derive(
//     Clone,
//     PartialEq,
//     Eq,
//     ::prost::Message,
//     serde::Serialize,
//     serde::Deserialize,
//     schemars::JsonSchema,
// )]
// pub struct IbcCounterpartyHeight {
//     #[prost(uint64, optional, tag = "1")]
//     revision_number: Option<u64>,
//     #[prost(uint64, optional, tag = "2")]
//     revision_height: Option<u64>,
// }

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
    StargateModule, // StargateAccepting, // CustomStargateKeeper<Empty, Empty, Empty>,
>;
