use crate::{
    admin_logic::admin,
    state::{OrbitalDomainConfig, UserConfig, USER_NONCE},
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
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, Uint64,
};

pub const CONTRACT_NAME: &str = "orbital-core";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type QueryDeps<'a> = Deps<'a, NeutronQuery>;
pub type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;

#[entry_point]
pub fn instantiate(
    deps: ExecuteDeps,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    USER_NONCE.save(deps.storage, &Uint64::zero())?;
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
            admin::try_update_ownership(deps, &env.block, &info.sender, action)
        }
        // admin action to enable new domain for user registration
        ExecuteMsg::RegisterNewDomain {
            domain,
            account_type,
        } => admin::try_register_new_domain(deps, info, domain, account_type),
        // user action to create a new user account which enables registration to domains
        ExecuteMsg::RegisterUser {} => user::try_register(deps, env, info),
        // user action to register a new domain which creates their clearing account
        ExecuteMsg::RegisterUserDomain { domain } => {
            user::try_register_new_domain(deps, env, info, domain)
        }
    }
}

#[entry_point]
pub fn query(deps: QueryDeps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OrbitalDomain { domain } => to_json_binary(&query_orbital_domain(deps, domain)?),
        QueryMsg::UserConfig { addr } => to_json_binary(&query_user_config(deps, addr)?),
        QueryMsg::Ownership {} => to_json_binary(&query_ownership(deps)?),
        QueryMsg::ClearingAccountAddress { addr, domain } => {
            to_json_binary(&query_clearing_account(deps, domain, addr)?)
        }
    }
}

fn query_clearing_account(
    deps: QueryDeps,
    domain: String,
    addr: String,
) -> StdResult<Option<String>> {
    let user_config = USER_CONFIGS.load(deps.storage, addr)?;
    let ica_id = get_ica_identifier(user_config.id, domain);
    CLEARING_ACCOUNTS.load(deps.storage, ica_id)
}

fn query_ownership(deps: QueryDeps) -> StdResult<cw_ownable::Ownership<Addr>> {
    get_ownership(deps.storage)
}

fn query_orbital_domain(deps: QueryDeps, domain: String) -> StdResult<OrbitalDomainConfig> {
    ORBITAL_DOMAINS.load(deps.storage, domain)
}

fn query_user_config(deps: QueryDeps, user: String) -> StdResult<UserConfig> {
    USER_CONFIGS.load(deps.storage, user)
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

    // Update the storage record associated with the interchain account.
    CLEARING_ACCOUNTS.save(deps.storage, ica_identifier, &Some(parsed_version.address))?;

    Ok(Response::default())
}
