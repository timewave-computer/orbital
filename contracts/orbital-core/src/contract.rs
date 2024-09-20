use crate::{
    admin_logic::admin,
    user_logic::user,
    utils::{extract_ica_identifier_from_port, get_ica_identifier, OpenAckVersion},
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cw_ownable::{get_ownership, initialize_owner};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    sudo::msg::SudoMsg,
    NeutronResult,
};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{CLEARING_ACCOUNTS, ORBITAL_DOMAINS, USER_CONFIGS},
};
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult,
};

pub const CONTRACT_NAME: &str = "orbital-core";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type OrbitalResult = NeutronResult<Response<NeutronMsg>>;
pub type QueryDeps<'a> = Deps<'a, NeutronQuery>;
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
pub fn execute(
    deps: ExecuteDeps,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        // admin action to manage ownership of orbital-core
        ExecuteMsg::UpdateOwnership(action) => {
            admin::transfer_admin(deps, &env.block, &info.sender, action)
        }
        // admin action to enable new domain for user registration
        ExecuteMsg::RegisterNewDomain {
            domain,
            account_type,
        } => admin::register_new_domain(deps, info, domain, account_type),
        // user action to create a new user account which enables registration to domains
        ExecuteMsg::RegisterUser {} => user::register(deps, env, info),
        // user action to register a new domain which creates their clearing account
        ExecuteMsg::RegisterUserDomain { domain } => {
            user::register_new_domain(deps, env, info, domain)
        }
    }
}

#[entry_point]
pub fn query(deps: QueryDeps, _env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::OrbitalDomain { domain } => query_orbital_domain(deps, domain),
        QueryMsg::UserConfig { addr } => query_user_config(deps, addr),
        QueryMsg::Ownership {} => query_ownership(deps),
        QueryMsg::UserRegisteredDomains { addr } => query_registered_domains(deps, addr),
        QueryMsg::ClearingAccountAddress { addr, domain } => {
            query_clearing_account(deps, domain, addr)
        }
    }
}

fn query_registered_domains(deps: QueryDeps, addr: String) -> NeutronResult<Binary> {
    let user_config = USER_CONFIGS.load(deps.storage, addr)?;
    Ok(to_json_binary(&user_config.registered_domains)?)
}

fn query_clearing_account(deps: QueryDeps, domain: String, addr: String) -> NeutronResult<Binary> {
    let ica_id = get_ica_identifier(addr, domain);
    let clearing_account = CLEARING_ACCOUNTS.load(deps.storage, ica_id)?;
    Ok(to_json_binary(&clearing_account)?)
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
    let user_config = USER_CONFIGS.load(deps.storage, user)?;
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
pub fn sudo(deps: ExecuteDeps, env: Env, msg: SudoMsg) -> StdResult<Response<NeutronMsg>> {
    match msg {
        // For handling successful registering of ICA
        SudoMsg::OpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        } => sudo_open_ack(
            deps,
            env,
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        ),
        _ => Ok(Response::default()),
    }
}

// handler
fn sudo_open_ack(
    deps: ExecuteDeps,
    _env: Env,
    port_id: String,
    _channel_id: String,
    _counterparty_channel_id: String,
    counterparty_version: String,
) -> StdResult<Response<NeutronMsg>> {
    // parse the response
    let parsed_version: OpenAckVersion =
        serde_json_wasm::from_str(counterparty_version.as_str())
            .map_err(|_| StdError::generic_err("Can't parse counterparty_version"))?;

    // extract the ICA identifier from the port
    let ica_identifier = extract_ica_identifier_from_port(port_id)?;
    println!("ica identifier: {ica_identifier}");
    // Update the storage record associated with the interchain account.
    CLEARING_ACCOUNTS.save(deps.storage, ica_identifier, &Some(parsed_version.address))?;

    Ok(Response::default())
}
