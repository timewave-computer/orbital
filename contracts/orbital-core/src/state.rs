use std::str::FromStr;

use crate::{contract::ExecuteDeps, error::ContractError};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coins, ensure, Addr, Binary, GrpcQuery, MessageInfo, Uint128, Uint64};
use cw_storage_plus::Map;
use cw_utils::must_pay;
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    interchain_txs::helpers::decode_message_response,
    proto_types::neutron::interchaintxs::v1::QueryParamsResponse,
};

/// map of users with their respective configurations
pub const USER_CONFIGS: Map<String, UserConfig> = Map::new("user_configs");

/// map of registered remote domains and their configuration
pub const ORBITAL_DOMAINS: Map<String, OrbitalDomainConfig> = Map::new("domains");

/// map of clearing accounts registered with orbital.
/// key is a composite of (domain_identifier, owner_neutron_addr).
/// value is an optional address where:
/// - None: clearing account is being registered and awaiting callback
/// - Some: clearing account has been registered and is ready for use
pub const CLEARING_ACCOUNTS: Map<(String, String), Option<Addr>> = Map::new("clearing_accounts");

#[cw_serde]
#[derive(Default)]
pub struct UserConfig {
    pub registered_domains: Vec<String>,
}

/// remote domain configuration config which supports different types of account implementations.
/// currently supported types:
/// - Polytone: cw-based account implementation that operates via note contract on the origin chain
/// - InterchainAccount: interchain account implementation based on ICS-27
#[cw_serde]
pub enum OrbitalDomainConfig {
    Polytone {
        note: Addr,
        timeout: Uint64,
    },
    InterchainAccount {
        connection_id: String,
        channel_id: String,
        timeout: Uint64,
    },
}

/// assumes that fees are only denominated in untrn and flattens them into a single coin
fn _flatten_ibc_fees_amt(fee_response: IbcFee) -> Uint128 {
    fee_response
        .ack_fee
        .iter()
        .chain(fee_response.recv_fee.iter())
        .chain(fee_response.timeout_fee.iter())
        .map(|fee| fee.amount)
        .sum()
}

fn assert_fee_payment(info: &MessageInfo, expected_fee: Uint128) -> Result<(), ContractError> {
    match must_pay(info, "untrn") {
        Ok(amt) => ensure!(
            amt >= expected_fee,
            ContractError::DomainRegistrationError("insufficient fee".to_string())
        ),
        Err(e) => return Err(ContractError::DomainRegistrationError(e.to_string())),
    };
    Ok(())
}

impl OrbitalDomainConfig {
    pub fn get_registration_message(
        &self,
        deps: ExecuteDeps,
        info: &MessageInfo,
    ) -> Result<NeutronMsg, ContractError> {
        match self {
            OrbitalDomainConfig::InterchainAccount { connection_id, .. } => {
                let grpc_query_msg = GrpcQuery {
                    path: "/neutron.interchaintxs.v1.Query/Params".to_string(),
                    data: Binary::new(Vec::new()),
                };
                let grpc_query_response: Binary = deps
                    .as_ref()
                    .querier
                    .query_grpc(grpc_query_msg.path.to_string(), grpc_query_msg.data.clone())?;

                let slice = grpc_query_response.to_vec();
                let query_params_response: QueryParamsResponse = decode_message_response(&slice)?;

                if let Some(params) = query_params_response.params {
                    for coin in params.register_fee.iter() {
                        let coin = Uint128::from_str(&coin.amount)?;
                        assert_fee_payment(info, coin)?;
                    }
                }

                Ok(NeutronMsg::register_interchain_account(
                    connection_id.to_string(),
                    info.sender.to_string(),
                    Some(coins(1, "untrn")),
                ))
            }
            _ => unimplemented!(),
        }
    }
}
