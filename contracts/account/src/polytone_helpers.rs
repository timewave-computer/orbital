use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{from_json, to_json_binary, Addr, Binary, CosmosMsg, DepsMut, Empty, Env, MessageInfo, QuerierWrapper, QueryRequest, Response, StdError, StdResult, Uint64, WasmMsg};
use neutron_sdk::{bindings::{msg::NeutronMsg, query::NeutronQuery}, NeutronError, NeutronResult};
use polytone::callbacks::{CallbackMessage, CallbackRequest, ErrorResponse};

use crate::state::{LEDGER, REGISTERED_NOTES, USER_DOMAINS};

use polytone::callbacks::{Callback as PolytoneCallback, ExecutionResponse};

type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;


pub const REGISTER_DOMAIN_CALLBACK_ID: u8 = 1;

pub fn try_handle_callback(
    env: Env,
    deps: ExecuteDeps,
    info: MessageInfo,
    msg: CallbackMessage,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg.result {
        PolytoneCallback::Query(resp) => process_query_callback(env, deps, resp, msg.initiator_msg),
        PolytoneCallback::Execute(resp) => {
            process_execute_callback(env, deps, resp, msg.initiator_msg, info)
        }
        PolytoneCallback::FatalError(resp) => process_fatal_error_callback(env, deps, resp),
    }
}

fn process_execute_callback(
    env: Env,
    deps: ExecuteDeps,
    execute_callback_result: Result<ExecutionResponse, String>,
    initiator_msg: Binary,
    info: MessageInfo,
) -> NeutronResult<Response<NeutronMsg>> {
    // only a registered note can submit a callback
    let note_domain = REGISTERED_NOTES.load(deps.storage, info.sender.clone())?;

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
            }
        }
        _ => (),
    }

    Ok(Response::default())
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


fn process_query_callback(
    env: Env,
    deps: ExecuteDeps,
    query_callback_result: Result<Vec<Binary>, ErrorResponse>,
    initiator_msg: Binary,
) -> NeutronResult<Response<NeutronMsg>> {
    // decode the initiator message callback id into u8
    let initiator_msg: u8 = from_json(initiator_msg)?;

    match initiator_msg {
        _ => Err(NeutronError::Std(StdError::generic_err("unexpected callback id".to_string()))),
    }
}

fn process_fatal_error_callback(
    env: Env,
    deps: ExecuteDeps,
    response: String,
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
