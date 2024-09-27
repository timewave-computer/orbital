use crate::{
    admin_logic::admin,
    icq::{self, Transfer, RECIPIENT_TXS, TRANSFERS},
    msg::GetTransfersAmountResponse,
    state::{ClearingAccountConfig, OrbitalDomainConfig, UserConfig, USER_NONCE},
    user_logic::user,
    utils::{extract_ica_identifier_from_port, get_ica_identifier, OpenAckVersion},
};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{CLEARING_ACCOUNTS, ORBITAL_DOMAINS, USER_CONFIGS},
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, Uint64,
};
use cw2::set_contract_version;
use cw_ownable::{get_ownership, initialize_owner};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_queries::v047::queries::{query_balance, BalanceResponse},
    sudo::msg::SudoMsg,
    NeutronResult,
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
        ExecuteMsg::UpdateOwnership(action) => {
            admin::try_update_ownership(deps, &env.block, &info.sender, action)
        }
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
        // user action to withdraw funds from a selected domain account they own
        ExecuteMsg::UserWithdrawFunds { domain, coin, dest } => {
            user::try_withdraw_from_remote_domain(deps, info, domain, coin, dest)
        }
        ExecuteMsg::RegisterBalancesQuery {
            connection_id,
            update_period,
            addr,
            denoms,
        } => icq::register_balances_query(connection_id, addr, denoms, update_period),
        ExecuteMsg::RegisterTransfersQuery {
            connection_id,
            update_period,
            recipient,
            min_height,
        } => icq::register_transfers_query(connection_id, recipient, update_period, min_height),
    }
}

#[entry_point]
pub fn query(deps: QueryDeps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OrbitalDomain { domain } => to_json_binary(&query_orbital_domain(deps, domain)?),
        QueryMsg::UserConfig { addr } => to_json_binary(&query_user_config(deps, addr)?),
        QueryMsg::Ownership {} => to_json_binary(&query_ownership(deps)?),
        QueryMsg::ClearingAccountAddress { addr, domain } => {
            to_json_binary(&query_clearing_account(deps, domain, addr)?)
        }
        QueryMsg::Balance { query_id } => to_json_binary(&query_icq_balance(deps, env, query_id)?),
        QueryMsg::IcqTransfer {} => to_json_binary(&query_transfers_number(deps)?),
        QueryMsg::IcqRecipientTxs { recipient } => {
            to_json_binary(&query_recipient_txs(deps, recipient)?)
        }
    }
}

fn query_recipient_txs(deps: QueryDeps, recipient: String) -> StdResult<Vec<Transfer>> {
    let txs = RECIPIENT_TXS.load(deps.storage, &recipient)?;
    Ok(txs)
}

fn query_transfers_number(deps: QueryDeps) -> StdResult<GetTransfersAmountResponse> {
    let transfers_number = TRANSFERS.load(deps.storage).unwrap_or_default();
    Ok(GetTransfersAmountResponse { transfers_number })
}

fn query_icq_balance(deps: QueryDeps, env: Env, query_id: u64) -> StdResult<BalanceResponse> {
    query_balance(deps, env, query_id).map_err(|e| StdError::generic_err(e.to_string()))
}

fn query_clearing_account(
    deps: QueryDeps,
    domain: String,
    addr: String,
) -> StdResult<Option<ClearingAccountConfig>> {
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
    unimplemented!()
}

#[entry_point]
pub fn migrate(_deps: ExecuteDeps, _env: Env, _msg: MigrateMsg) -> StdResult<Response<NeutronMsg>> {
    unimplemented!()
}

// neutron uses the `sudo` entry point in their ICA/ICQ related logic
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
        // For handling tx query result
        SudoMsg::TxQueryResult {
            query_id,
            height,
            data,
        } => icq::sudo_tx_query_result(deps, env, query_id, height, data),

        // For handling kv query result
        SudoMsg::KVQueryResult { query_id } => icq::sudo_kv_query_result(deps, env, query_id),
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

    let clearing_account_config = ClearingAccountConfig {
        addr: parsed_version.address,
        controller_connection_id: parsed_version.controller_connection_id,
    };

    // Update the storage record associated with the interchain account.
    CLEARING_ACCOUNTS.save(deps.storage, ica_identifier, &Some(clearing_account_config))?;

    Ok(Response::default())
}
