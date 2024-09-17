use cosmwasm_schema::cw_serde;

pub mod custom_keepers;
pub mod custom_module;

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

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTransfer {
    /// the port on which the packet will be sent
    #[prost(string, tag = "1")]
    pub source_port: String,
    /// the channel by which the packet will be sent
    #[prost(string, tag = "2")]
    pub source_channel: String,
    /// the tokens to be transferred
    #[prost(message, optional, tag = "3")]
    pub token: Option<cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
    /// the sender address
    #[prost(string, tag = "4")]
    pub sender: String,
    /// the recipient address on the destination chain
    #[prost(string, tag = "5")]
    pub receiver: String,
    /// Timeout height relative to the current block height.
    /// The timeout is disabled when set to 0.
    #[prost(message, optional, tag = "6")]
    pub timeout_height: Option<IbcCounterpartyHeight>,
    /// Timeout timestamp in absolute nanoseconds since unix epoch.
    /// The timeout is disabled when set to 0.
    #[prost(uint64, tag = "7")]
    pub timeout_timestamp: u64,
    #[prost(string, tag = "8")]
    pub memo: String,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
pub struct IbcCounterpartyHeight {
    #[prost(uint64, optional, tag = "1")]
    revision_number: Option<u64>,
    #[prost(uint64, optional, tag = "2")]
    revision_height: Option<u64>,
}
