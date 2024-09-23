use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Coin, MessageInfo, StdError, StdResult, Uint128, Uint64};
use cw_utils::must_pay;
use neutron_sdk::bindings::msg::IbcFee;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;

pub fn assert_fee_payment(info: &MessageInfo, expected_fee: &Coin) -> Result<(), ContractError> {
    match must_pay(info, &expected_fee.denom) {
        Ok(amt) => ensure!(
            amt >= expected_fee.amount,
            ContractError::DomainRegistrationError("insufficient fee".to_string())
        ),
        Err(_) => {
            return Err(ContractError::DomainRegistrationError(format!(
                "no funds sent; expected {}.",
                expected_fee
            )))
        }
    };
    Ok(())
}

/// assumes that fees are only denominated in untrn and flattens them into a single coin
pub fn _flatten_ibc_fees_amt(fee_response: IbcFee) -> Uint128 {
    fee_response
        .ack_fee
        .iter()
        .chain(fee_response.recv_fee.iter())
        .chain(fee_response.timeout_fee.iter())
        .map(|fee| fee.amount)
        .sum()
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

#[cw_serde]
pub struct AccountIdentifier {
    pub id: String,
}

impl AccountIdentifier {
    pub fn try_from_port(port: String) -> StdResult<AccountIdentifier> {
        Ok(AccountIdentifier {
            id: extract_ica_identifier_from_port(port)?,
        })
    }

    pub fn new(user_id: Uint64, domain: String) -> AccountIdentifier {
        AccountIdentifier {
            id: get_ica_identifier(user_id, domain),
        }
    }
}
