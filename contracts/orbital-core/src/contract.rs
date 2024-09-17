#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cw_ownable::{assert_owner, get_ownership, initialize_owner, update_ownership, Action};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    sudo::msg::SudoMsg,
    NeutronResult,
};

use crate::{
    account_types::UncheckedOrbitalDomainConfig,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{UserConfig, ORBITAL_DOMAINS, USER_CONFIGS},
};
use cosmwasm_std::{
    ensure, to_json_binary, Addr, Binary, BlockInfo, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult,
};

pub const CONTRACT_NAME: &str = "orbital-core";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type OrbitalResult = NeutronResult<Response<NeutronMsg>>;
type QueryDeps<'a> = Deps<'a, NeutronQuery>;
pub type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;

#[entry_point]
pub fn instantiate(
    deps: ExecuteDeps,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> OrbitalResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    Ok(Response::new())
}

#[entry_point]
pub fn execute(deps: ExecuteDeps, env: Env, info: MessageInfo, msg: ExecuteMsg) -> OrbitalResult {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            admin_update_ownership(deps, &env.block, &info.sender, action)
        }
        ExecuteMsg::RegisterNewDomain {
            domain,
            account_type,
        } => admin_register_new_domain(deps, info, domain, account_type),
        ExecuteMsg::RegisterUser {} => register_user(deps, env, info),
        ExecuteMsg::RegisterUserDomain { domain } => {
            register_new_user_domain(deps, env, info, domain)
        }
    }
}

fn register_new_user_domain(
    deps: ExecuteDeps,
    _env: Env,
    info: MessageInfo,
    domain: String,
) -> OrbitalResult {
    // user must be registered to operate on domains
    ensure!(
        USER_CONFIGS.has(deps.storage, info.sender.clone()),
        ContractError::UserNotRegistered {}
    );

    // the domain must be enabled on orbital level to be able to register
    ensure!(
        ORBITAL_DOMAINS.has(deps.storage, domain.to_string()),
        ContractError::UnknownDomain(domain)
    );

    let domain_config = ORBITAL_DOMAINS.load(deps.storage, domain.to_string())?;

    // TODO: query and assert registration fee payment for the domain (if applicable)

    // TODO: fire a registration message
    let registration_msg =
        domain_config.get_registration_message(deps, domain, info.sender.clone())?;

    Ok(Response::new()
        .add_message(registration_msg)
        .add_attribute("method", "register_user_domain"))
}

fn register_user(deps: ExecuteDeps, _env: Env, info: MessageInfo) -> OrbitalResult {
    // user can only register once
    ensure!(
        !USER_CONFIGS.has(deps.storage, info.sender.clone()),
        ContractError::UserAlreadyRegistered {}
    );

    // save an empty user config
    USER_CONFIGS.save(deps.storage, info.sender, &UserConfig::default())?;

    Ok(Response::new().add_attribute("method", "register_user"))
}

fn admin_update_ownership(
    deps: ExecuteDeps,
    block: &BlockInfo,
    sender: &Addr,
    action: Action,
) -> OrbitalResult {
    let resp = update_ownership(deps.into_empty(), block, sender, action)
        .map_err(ContractError::Ownership)?;
    Ok(Response::default().add_attributes(resp.into_attributes()))
}

fn admin_register_new_domain(
    deps: ExecuteDeps,
    info: MessageInfo,
    domain: String,
    account_type: UncheckedOrbitalDomainConfig,
) -> OrbitalResult {
    // only the owner can register new domains
    assert_owner(deps.storage, &info.sender).map_err(ContractError::Ownership)?;

    // validate the domain configuration
    let orbital_domain = account_type.try_into_checked(deps.api)?;

    // ensure the domain does not already exist
    ensure!(
        !ORBITAL_DOMAINS.has(deps.storage, domain.to_string()),
        ContractError::OrbitalDomainAlreadyExists(domain.to_string())
    );

    // store the validated domain config in state
    ORBITAL_DOMAINS.save(deps.storage, domain.to_string(), &orbital_domain)?;

    Ok(Response::default()
        .add_attribute("method", "register_new_domain")
        .add_attribute("domain", domain))
}

#[entry_point]
pub fn query(deps: QueryDeps, _env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::OrbitalDomain { domain } => query_orbital_domain(deps, domain),
        QueryMsg::UserConfig { user } => query_user_config(deps, user),
        QueryMsg::Ownership {} => query_ownership(deps),
    }
}

fn query_ownership(deps: QueryDeps) -> NeutronResult<Binary> {
    let ownership = get_ownership(deps.storage)?;
    Ok(to_json_binary(&ownership)?)
}

fn query_orbital_domain(deps: QueryDeps, domain: String) -> NeutronResult<Binary> {
    let domain_config = ORBITAL_DOMAINS.load(deps.storage, domain)?;
    Ok(to_json_binary(&domain_config)?)
}

fn query_user_config(deps: QueryDeps, user: String) -> NeutronResult<Binary> {
    let user_config = USER_CONFIGS.load(deps.storage, Addr::unchecked(user))?;
    Ok(to_json_binary(&user_config)?)
}

#[entry_point]
pub fn reply(_deps: ExecuteDeps, _env: Env, _msg: Reply) -> StdResult<Response<NeutronMsg>> {
    Ok(Response::default())
}

#[entry_point]
pub fn migrate(_deps: ExecuteDeps, _env: Env, _msg: MigrateMsg) -> StdResult<Response<NeutronMsg>> {
    Ok(Response::default())
}

#[entry_point]
pub fn sudo(_deps: ExecuteDeps, _env: Env, _msg: SudoMsg) -> StdResult<Response<NeutronMsg>> {
    Ok(Response::default())
}
