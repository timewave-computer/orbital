use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coins, ensure, to_json_vec, Addr, Binary, Coin, GrpcQuery, MessageInfo, QuerierWrapper,
    QueryRequest, Response, StdResult, Uint128, Uint64,
};
use cw_storage_plus::Map;
use cw_utils::must_pay;
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    NeutronResult,
};

use crate::{contract::ExecuteDeps, error::ContractError};

/// map of users with their respective configurations
pub const USER_CONFIGS: Map<Addr, UserConfig> = Map::new("user_configs");

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
pub struct UserConfig {}

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
fn flatten_ibc_fees_amt(fee_response: IbcFee) -> Uint128 {
    fee_response
        .ack_fee
        .iter()
        .chain(fee_response.recv_fee.iter())
        .chain(fee_response.timeout_fee.iter())
        .map(|fee| fee.amount)
        .sum()
}

pub fn get_ictxs_module_params_grpc_query_msg() -> GrpcQuery {
    GrpcQuery {
        path: "/neutron.interchaintxs.v1.Query/Params".to_string(),
        data: Binary::new(Vec::new()),
    }
}

pub fn get_ictxs_module_params_query_msg() -> QueryRequest<NeutronQuery> {
    QueryRequest::Stargate {
        path: "/neutron.interchaintxs.v1.Query/Params".to_string(),
        data: Binary::new(Vec::new()),
    }
}

pub fn query_ica_registration_fee(
    querier: QuerierWrapper<'_, NeutronQuery>,
) -> StdResult<Vec<Coin>> {
    let query_msg = get_ictxs_module_params_query_msg();
    println!("query message: {:?}", query_msg);
    let response: QueryParamsResponse = querier.query(&query_msg)?;
    println!("response: {:?}", response);
    Ok(response.params.unwrap().register_fee)
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

#[cw_serde]
pub struct QueryParamsResponse {
    pub params: Option<Params>,
}

#[cw_serde]
pub struct Params {
    pub msg_submit_tx_max_messages: Uint64,
    pub register_fee: Vec<Coin>,
}

impl OrbitalDomainConfig {
    pub fn get_registration_message(
        &self,
        deps: ExecuteDeps,
        info: &MessageInfo,
        domain: String,
    ) -> StdResult<NeutronMsg> {
        let msg = match self {
            OrbitalDomainConfig::InterchainAccount { connection_id, .. } => {
                println!("getting registration message\n\n");
                // match deps.as_ref().into_empty().querier.query_grpc(
                //     "/neutron.interchaintxs.v1.Query/Params".to_string(),
                //     Binary::new(Vec::new()),
                // ) {
                //     Ok(r) => println!("grpc query success: {:?}", r),
                //     Err(e) => println!("grpc query error: {:?}", e),
                // };
                let grpc_query_msg = get_ictxs_module_params_grpc_query_msg();
                let query_request: QueryRequest<NeutronQuery> = QueryRequest::Grpc(grpc_query_msg);
                let resp = deps
                    .querier
                    .into_empty()
                    .raw_query(&to_json_vec(&query_request)?);
                println!("response: {:?}", resp);
                NeutronMsg::register_interchain_account(
                    connection_id.to_string(),
                    info.sender.to_string(),
                    Some(coins(1, "untrn")),
                )
            }
            _ => unimplemented!(),
        };

        // store `None` as the clearing account until the callback is received
        CLEARING_ACCOUNTS.save(deps.storage, (domain, info.sender.to_string()), &None)?;

        Ok(msg)
    }
}
