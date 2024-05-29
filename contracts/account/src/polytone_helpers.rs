use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    ensure, from_json, to_json_binary, Addr, AllBalanceResponse, BankQuery, Binary, CosmosMsg,
    DepsMut, Empty, Env, MessageInfo, QuerierWrapper, QueryRequest, Response, StdError, StdResult,
    Uint64, WasmMsg,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronError, NeutronResult,
};
use orbital_utils::domain::OrbitalDomain;
use polytone::callbacks::{CallbackMessage, CallbackRequest, ErrorResponse};

use crate::state::{LEDGER, NOTE_TO_DOMAIN, USER_DOMAINS};

use polytone::callbacks::{Callback as PolytoneCallback, ExecutionResponse};

type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;

pub const REGISTER_DOMAIN_CALLBACK_ID: u8 = 1;
pub const SYNC_DOMAIN_CALLBACK_ID: u8 = 2;

pub fn try_handle_callback(
    env: Env,
    deps: ExecuteDeps,
    info: MessageInfo,
    msg: CallbackMessage,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg.result {
        PolytoneCallback::Query(resp) => {
            process_query_callback(env, deps, info, resp, msg.initiator_msg)
        }
        PolytoneCallback::Execute(resp) => {
            process_execute_callback(env, deps, resp, msg.initiator_msg, info)
        }
        PolytoneCallback::FatalError(resp) => process_fatal_error_callback(env, deps, resp),
    }
}

pub fn get_proxy_query_balances_message(
    env: Env,
    proxy_address: String,
    note_address: String,
) -> StdResult<WasmMsg> {
    let bal_query_request: QueryRequest<Empty> = BankQuery::AllBalances {
        address: proxy_address,
    }
    .into();

    let query_msg = PolytoneExecuteMsg::Query {
        msgs: vec![bal_query_request],
        callback: CallbackRequest {
            msg: to_json_binary(&SYNC_DOMAIN_CALLBACK_ID)?,
            receiver: env.contract.address.to_string(),
        },
        timeout_seconds: Uint64::new(120),
    };

    Ok(WasmMsg::Execute {
        contract_addr: note_address.to_string(),
        msg: to_json_binary(&query_msg)?,
        funds: vec![],
    })
}

fn process_execute_callback(
    env: Env,
    deps: ExecuteDeps,
    execute_callback_result: Result<ExecutionResponse, String>,
    initiator_msg: Binary,
    info: MessageInfo,
) -> NeutronResult<Response<NeutronMsg>> {
    // only a registered note can submit a callback
    let note_domain = NOTE_TO_DOMAIN.load(deps.storage, info.sender.clone())?;

    let _callback_result: ExecutionResponse = match execute_callback_result {
        Ok(val) => val,
        Err(e) => return Err(NeutronError::Std(StdError::generic_err(e.to_string()))),
    };

    match from_json(initiator_msg)? {
        REGISTER_DOMAIN_CALLBACK_ID => {
            let proxy_address = query_polytone_proxy_address(
                env.contract.address.to_string(),
                info.sender.to_string(),
                deps.querier,
            )?;

            if let Some(addr) = proxy_address {
                USER_DOMAINS.save(deps.storage, note_domain.value(), &addr)?;
                LEDGER.save(deps.storage, note_domain.value(), &HashMap::new())?;
            } else {
                let debug = format!("process_execute_callback [REGISTER_DOMAIN_CALLBACK_ID]: {:?}", proxy_address);
                USER_DOMAINS.save(deps.storage, note_domain.value(), &debug)?;
            }
        }
        _ => {
            let debug = format!("process_execute_callback [_]: {:?}", _callback_result);
            USER_DOMAINS.save(deps.storage, note_domain.value(), &debug)?;
        },
    }

    Ok(Response::default())
}

fn process_query_callback(
    env: Env,
    deps: ExecuteDeps,
    info: MessageInfo,
    query_callback_result: Result<Vec<Binary>, ErrorResponse>,
    initiator_msg: Binary,
) -> NeutronResult<Response<NeutronMsg>> {
    // only a registered note can submit a callback
    let note_domain = NOTE_TO_DOMAIN.load(deps.storage, info.sender.clone())?;
    let mut ledger = LEDGER.load(deps.storage, note_domain.value())?;
    let domain_log = format!("handle_domain_balances_sync_callback domain: {:?}", note_domain.value());
    ledger.insert(domain_log, 0);
    LEDGER.save(deps.storage, note_domain.value(), &ledger)?;

    match from_json(initiator_msg)? {
        SYNC_DOMAIN_CALLBACK_ID => {
            handle_domain_balances_sync_callback(deps, env, query_callback_result, note_domain)
        }
        _ => {
            let mut ledger = LEDGER.load(deps.storage, note_domain.value())?;
            let domain_log = format!("handle_domain_balances_sync_callback domain: {:?}", note_domain.value());
            ledger.insert(domain_log, 0);
            LEDGER.save(deps.storage, note_domain.value(), &ledger)?;
            Ok(Response::default())
        },
    }

}

fn handle_domain_balances_sync_callback(
    deps: ExecuteDeps,
    _env: Env,
    query_callback_result: Result<Vec<Binary>, ErrorResponse>,
    domain: OrbitalDomain,
) -> NeutronResult<Response<NeutronMsg>> {
    let mut ledger = LEDGER.load(deps.storage, domain.value())?;

    let response_binary = match query_callback_result {
        Ok(val) => val,
        Err(_) => {
            let domain_log = format!("query_callback_result {:?}", query_callback_result);
            ledger.insert(domain_log, 0);
            return Ok(Response::default())
        }
    };

    let domain_log = format!("handle_domain_balances_sync_callback domain: {:?}", domain.value());
    ledger.insert(domain_log, 0);

    match from_json(&response_binary[0])? {
        AllBalanceResponse { amount } => {
            for coin in amount {
                ledger.insert(coin.denom, coin.amount.u128());
            }
        }
        _ => {
            let log = format!("failed to from_json the response binary: {:?}", response_binary);
            ledger.insert(log, 0);
        }
    };

    LEDGER.save(deps.storage, domain.value(), &ledger)?;
    Ok(Response::default())
}

fn process_fatal_error_callback(
    _env: Env,
    _deps: ExecuteDeps,
    _response: String,
) -> NeutronResult<Response<NeutronMsg>> {
    Ok(Response::default())
}

#[cw_serde]
pub enum PolytoneExecuteMsg {
    Query {
        msgs: Vec<QueryRequest<Empty>>,
        callback: CallbackRequest,
        timeout_seconds: Uint64,
    },
    Execute {
        msgs: Vec<CosmosMsg<Empty>>,
        callback: Option<CallbackRequest>,
        timeout_seconds: Uint64,
    },
}

pub fn get_note_execute_neutron_msg(
    msgs: Vec<CosmosMsg>,
    ibc_timeout: Uint64,
    note_address: Addr,
    callback: Option<CallbackRequest>,
) -> NeutronResult<CosmosMsg<NeutronMsg>> {
    let polytone_msg = get_polytone_execute_msg_binary(msgs, callback, ibc_timeout)?;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: note_address.to_string(),
        msg: polytone_msg,
        funds: vec![],
    }))
}

pub fn get_note_query_neutron_msg(
    msgs: Vec<QueryRequest<Empty>>,
    ibc_timeout: Uint64,
    note_address: Addr,
    callback: CallbackRequest,
) -> NeutronResult<CosmosMsg<NeutronMsg>> {
    let polytone_msg = get_polytone_query_msg_binary(msgs, ibc_timeout, callback)?;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: note_address.to_string(),
        msg: polytone_msg,
        funds: vec![],
    }))
}

pub fn get_polytone_query_msg_binary(
    msgs: Vec<QueryRequest<Empty>>,
    timeout_seconds: Uint64,
    callback: CallbackRequest,
) -> StdResult<Binary> {
    let query_msg = PolytoneExecuteMsg::Query {
        msgs,
        callback,
        timeout_seconds,
    };
    to_json_binary(&query_msg)
}

pub fn get_polytone_execute_msg_binary(
    msgs: Vec<CosmosMsg>,
    callback: Option<CallbackRequest>,
    timeout_seconds: Uint64,
) -> StdResult<Binary> {
    let execute_msg = PolytoneExecuteMsg::Execute {
        msgs,
        callback,
        timeout_seconds,
    };
    to_json_binary(&execute_msg)
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum PolytoneQueryMsg {
    #[returns(Option<String>)]
    RemoteAddress { local_address: String },
    #[returns(Uint64)]
    BlockMaxGas,
}

pub fn query_polytone_proxy_address(
    local_address: String,
    note_address: String,
    querier: QuerierWrapper<NeutronQuery>,
) -> Result<Option<String>, StdError> {
    let remote_address_query = PolytoneQueryMsg::RemoteAddress { local_address };

    querier.query_wasm_smart(note_address, &remote_address_query)
}
