use cosmwasm_std::{Binary, StdError, StdResult, Uint64};
use neutron_sdk::{bindings::types::ProtobufAny, NeutronResult};
use prost::Message;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod fees {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{
        ensure, Binary, Coin, MessageInfo, QueryRequest, StdResult, Uint128, Uint64,
    };
    use cw_utils::must_pay;
    use neutron_sdk::bindings::{msg::IbcFee, query::NeutronQuery};

    use crate::{contract::ExecuteDeps, error::ContractError};

    #[cw_serde]
    pub struct Params {
        pub msg_submit_tx_max_messages: Uint64,
        pub register_fee: Vec<Coin>,
    }

    #[cw_serde]
    pub struct QueryParamsResponse {
        pub params: Option<Params>,
    }

    pub fn assert_fee_payment(
        info: &MessageInfo,
        expected_fee: &Coin,
    ) -> Result<(), ContractError> {
        let paid_amt = must_pay(info, &expected_fee.denom)?;
        ensure!(
            paid_amt >= expected_fee.amount,
            ContractError::DomainRegistrationError("insufficient fee".to_string())
        );

        Ok(())
    }

    /// assumes that fees are only denominated in untrn and flattens them into a single coin
    pub fn flatten_ibc_fees_amt(fee_response: &IbcFee) -> Uint128 {
        fee_response
            .ack_fee
            .iter()
            .chain(fee_response.recv_fee.iter())
            .chain(fee_response.timeout_fee.iter())
            .map(|fee| fee.amount)
            .sum()
    }

    /// helper method to query the registration fee for the ICA
    pub fn query_ica_registration_fee(deps: ExecuteDeps) -> StdResult<QueryParamsResponse> {
        // TODO: remove this explicit allow
        #[allow(deprecated)]
        let stargate_query_msg: QueryRequest<NeutronQuery> = QueryRequest::Stargate {
            path: "/neutron.interchaintxs.v1.Query/Params".to_string(),
            data: Binary::default(),
        };

        let response: QueryParamsResponse = deps.querier.query(&stargate_query_msg)?;

        Ok(response)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) struct OpenAckVersion {
    pub version: String,
    pub controller_connection_id: String,
    pub host_connection_id: String,
    pub address: String,
    pub encoding: String,
    pub tx_type: String,
}

/// returns the ICA identifier for this specific (user, domain) combination.
/// it can be any string.
pub fn get_ica_identifier(user_id: Uint64, domain: String) -> String {
    let id = user_id.to_string();
    format!("{domain}{id}")
}

/// inverse of neutron_sdk::interchain_txs::helpers::get_port_id,
/// which turns string of format "icacontroller-{contract_address}.{interchain_account_id}".
/// returns the interchain_account_id substring.
pub fn extract_ica_identifier_from_port(port: String) -> StdResult<String> {
    let parts: Vec<&str> = port.split('.').collect();
    match parts.len() {
        2 => Ok(parts[1].to_string()),
        _ => Err(StdError::generic_err("invalid port id {port}".to_string())),
    }
}

pub fn generate_proto_msg(msg: impl Message, type_url: &str) -> NeutronResult<ProtobufAny> {
    let buf = msg.encode_to_vec();

    let any_msg = ProtobufAny {
        type_url: type_url.to_string(),
        value: Binary::from(buf),
    };
    Ok(any_msg)
}
