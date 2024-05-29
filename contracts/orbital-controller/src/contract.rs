use std::collections::HashMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use cw2::set_contract_version;
use orbital_utils::domain::OrbitalDomain;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{UserConfig, POLYTONE_NOTES, USER_CONFIGS},
};

const CONTRACT_NAME: &str = "crates.io:vesting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterUser { domains } => execute_register_user(deps, env, info, domains),
        ExecuteMsg::RegisterUserDomain { domain } => {
            execute_register_user_domain(deps, env, info, domain)
        }
        ExecuteMsg::RegisterDomain { domain, note_addr } => {
            execute_register_domain(deps, env, info, domain, note_addr)
        }
    }
}

/// registers user to the orbital system
pub fn execute_register_user(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    domains: Vec<OrbitalDomain>,
) -> Result<Response, ContractError> {
    let mut registered_domains = HashMap::new();

    // this should submit a transaction to the polytone contract to create a new polytone for the user.
    // polytone callback should update this value to the instantiated proxy address.
    for domain in domains {
        registered_domains.insert(
            domain.value(),
            Addr::unchecked("unique user polytone".to_string()),
        );
    }

    let user_config = UserConfig { registered_domains };

    USER_CONFIGS.save(deps.storage, info.sender, &user_config)?;

    Ok(Response::new())
}

pub fn execute_register_user_domain(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _domain: OrbitalDomain,
) -> Result<Response, ContractError> {
    // registers a new user domain
    // instantiates a polytone for the user for the domain
    Ok(Response::new())
}

/// registers a new domain to the system by storing the polytone note address.
pub fn execute_register_domain(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    domain: OrbitalDomain,
    note_addr: String,
) -> Result<Response, ContractError> {
    let note = deps.api.addr_validate(&note_addr)?;
    POLYTONE_NOTES.save(deps.storage, domain.value(), &note)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetClaimable {} => Ok(to_json_binary(&())?),
    }
}
