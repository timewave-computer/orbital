use std::collections::HashMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64};

use cw2::set_contract_version;
use neutron_sdk::{bindings::{msg::NeutronMsg, query::NeutronQuery}, NeutronResult};
use orbital_utils::domain::OrbitalDomain;
use polytone::callbacks::CallbackRequest;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    polytone_helpers::{get_note_execute_neutron_msg, try_handle_callback, REGISTER_DOMAIN_CALLBACK_ID},
    state::{LEDGER, REGISTERED_NOTES, USER_DOMAINS}
};

const CONTRACT_NAME: &str = "crates.io:vesting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: ExecuteDeps,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // we initialize an empty ledger for the user to enable fund deposits/withdrawals
    LEDGER.save(deps.storage, OrbitalDomain::Neutron.value(), &HashMap::new())?;

    // root domain address is this contract
    USER_DOMAINS.save(deps.storage, OrbitalDomain::Neutron.value(), &env.contract.address.to_string())?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: ExecuteDeps,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterDomain {
            domain,
            note_addr,
        } => execute_register_domain(deps, env, info, domain, note_addr),
        ExecuteMsg::Callback(callback_msg) => try_handle_callback(env, deps, info, callback_msg),
    }
}

pub fn execute_register_domain(
    deps: ExecuteDeps,
    env: Env,
    info: MessageInfo,
    domain: OrbitalDomain,
    note_addr: String,
) -> NeutronResult<Response<NeutronMsg>>  {

    // validate the note address (on neutron chain)
    let note = deps.api.addr_validate(&note_addr)?;
    REGISTERED_NOTES.save(deps.storage, note.clone(), &domain)?;

    // get the polytone init message and send it out
    let polytone_init_msg = get_note_execute_neutron_msg(
        vec![],
        Uint64::new(120),
        note,
        Some(CallbackRequest {
            receiver: env.contract.address.to_string(),
            msg: to_json_binary(&REGISTER_DOMAIN_CALLBACK_ID)?,
        }),
    )?;

    Ok(Response::new().add_message(polytone_init_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
    }
}
