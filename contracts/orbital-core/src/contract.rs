#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cw_ownable::{
    assert_owner, get_ownership, initialize_owner, update_ownership, Action, OwnershipError,
};

use crate::{
    account_types::UncheckedOrbitalDomainConfig,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::ORBITAL_DOMAINS,
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, BlockInfo, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

pub const CONTRACT_NAME: &str = "orbital-core";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, OwnershipError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            admin_update_ownership(deps, &env.block, &info.sender, action)
        }
        ExecuteMsg::RegisterNewDomain {
            domain,
            account_type,
        } => admin_register_new_domain(deps, info, domain, account_type),
    }
}

fn admin_update_ownership(
    deps: DepsMut,
    block: &BlockInfo,
    sender: &Addr,
    action: Action,
) -> Result<Response, ContractError> {
    let resp = update_ownership(deps, block, sender, action)?;
    Ok(Response::default().add_attributes(resp.into_attributes()))
}

fn admin_register_new_domain(
    deps: DepsMut,
    info: MessageInfo,
    domain: String,
    account_type: UncheckedOrbitalDomainConfig,
) -> Result<Response, ContractError> {
    // only the owner can register new domains
    assert_owner(deps.storage, &info.sender)?;

    // validate the domain configuration
    let orbital_domain = account_type.try_into_checked(&deps)?;

    // ensure the domain does not already exist
    if ORBITAL_DOMAINS.has(deps.storage, domain.to_string()) {
        return Err(ContractError::OrbitalDomainAlreadyExists(
            domain.to_string(),
        ));
    }

    // store the validated domain config in state
    ORBITAL_DOMAINS.save(deps.storage, domain.to_string(), &orbital_domain)?;

    Ok(Response::default()
        .add_attribute("method", "register_new_domain")
        .add_attribute("domain", domain))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_json_binary(&get_ownership(deps.storage)?),
        QueryMsg::OrbitalDomain { domain } => query_orbital_domain(deps, domain),
    }
}

fn query_orbital_domain(deps: Deps, domain: String) -> StdResult<Binary> {
    let domain_config = ORBITAL_DOMAINS.load(deps.storage, domain)?;
    to_json_binary(&domain_config)
}
