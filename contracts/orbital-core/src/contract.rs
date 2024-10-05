use crate::{
    admin_logic::admin,
    icq::{self},
    msg::{GetTransfersAmountResponse, RecipientTxsResponse},
    state::{
        ClearingAccountConfig, OrbitalAuctionConfig, OrbitalDomainConfig, UserConfig,
        ORBITAL_AUCTIONS, ORBITAL_AUCTION_CODE_ID, ORBITAL_AUCTION_NONCE, RECIPIENT_TXS, TRANSFERS,
        USER_NONCE,
    },
    user_logic::user,
    utils::{ClearingIcaIdentifier, OpenAckVersion},
};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{CLEARING_ACCOUNTS, ORBITAL_DOMAINS, USER_CONFIGS},
};

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
    NeutronError, NeutronResult,
};

pub const CONTRACT_NAME: &str = "orbital-core";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type QueryDeps<'a> = Deps<'a, NeutronQuery>;
pub type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: ExecuteDeps,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    USER_NONCE.save(deps.storage, &Uint64::zero())?;
    ORBITAL_AUCTION_CODE_ID.save(deps.storage, &msg.auction_code_id)?;
    ORBITAL_AUCTION_NONCE.save(deps.storage, &Uint64::zero())?;

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
        ExecuteMsg::UpdateOwnership(action) => {
            admin::try_update_ownership(deps, &env.block, &info.sender, action)
        }
        ExecuteMsg::RegisterNewDomain {
            domain,
            account_type,
        } => admin::try_register_new_domain(deps, info, domain, account_type),
        ExecuteMsg::RegisterNewAuction(instantiate_msg) => {
            admin::try_register_new_auction(deps, info, instantiate_msg)
        }
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: QueryDeps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OrbitalDomain { domain } => to_json_binary(&query_orbital_domain(deps, domain)?),
        QueryMsg::UserConfig { addr } => to_json_binary(&query_user_config(deps, addr)?),
        QueryMsg::Ownership {} => to_json_binary(&query_ownership(deps)?),
        QueryMsg::ClearingAccountAddress { addr, domain } => {
            to_json_binary(&query_clearing_account(deps, domain, addr)?)
        }
        QueryMsg::Balance { query_id } => to_json_binary(&query_icq_balance(deps, env, query_id)?),
        QueryMsg::IcqTransfersAmount {} => to_json_binary(&query_transfers_number(deps)?),
        QueryMsg::IcqRecipientTxs { recipient } => {
            to_json_binary(&query_recipient_txs(deps, recipient)?)
        }
        QueryMsg::Auction { id } => to_json_binary(&query_auction_by_id(deps, id)?),
        QueryMsg::AuctionClearingAccountAddress { id, domain } => {
            to_json_binary(&query_auction_clearing_account(deps, id, domain)?)
        }
    }
}

fn query_auction_by_id(deps: QueryDeps, auction_id: Uint64) -> StdResult<OrbitalAuctionConfig> {
    let auction = ORBITAL_AUCTIONS.load(deps.storage, auction_id.u64())?;

    Ok(auction)
}

fn query_recipient_txs(deps: QueryDeps, recipient: String) -> StdResult<RecipientTxsResponse> {
    let txs = RECIPIENT_TXS
        .may_load(deps.storage, recipient)?
        .unwrap_or_default();

    Ok(RecipientTxsResponse { transfers: txs })
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
    let user_clearing_account = ClearingIcaIdentifier::User {
        user_id: user_config.id.u64(),
        domain,
    };

    CLEARING_ACCOUNTS.load(deps.storage, user_clearing_account.to_str_identifier())
}

fn query_auction_clearing_account(
    deps: QueryDeps,
    auction_id: Uint64,
    domain: String,
) -> StdResult<Option<ClearingAccountConfig>> {
    let auction_clearing_account = ClearingIcaIdentifier::Auction {
        auction_id: auction_id.u64(),
        domain,
    };

    CLEARING_ACCOUNTS.load(deps.storage, auction_clearing_account.to_str_identifier())
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: ExecuteDeps, _env: Env, _msg: Reply) -> StdResult<Response<NeutronMsg>> {
    Err(StdError::generic_err("unimplemented!()"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: ExecuteDeps, _env: Env, _msg: MigrateMsg) -> StdResult<Response<NeutronMsg>> {
    Err(StdError::generic_err("unimplemented!()"))
}

// neutron uses the `sudo` entry point in their ICA/ICQ related logic
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: ExecuteDeps, env: Env, msg: SudoMsg) -> NeutronResult<Response<NeutronMsg>> {
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
        )
        .map_err(NeutronError::Std),
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
    env: Env,
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
    let parsed_ica_identifier = ClearingIcaIdentifier::from_str_identifier(&port_id)?;

    let string_ica_identifier = parsed_ica_identifier.to_str_identifier();

    let clearing_account_config = ClearingAccountConfig {
        addr: parsed_version.address,
        controller_connection_id: parsed_version.controller_connection_id,
    };

    // Update the storage record associated with the interchain account.
    CLEARING_ACCOUNTS.save(
        deps.storage,
        string_ica_identifier.to_string(),
        &Some(clearing_account_config.clone()),
    )?;

    // if the clearing account is associated with an auction, we need to handle some hooks
    if let ClearingIcaIdentifier::Auction { auction_id, domain } = parsed_ica_identifier.clone() {
        let mut associated_orbital_auction = ORBITAL_AUCTIONS.load(deps.storage, auction_id)?;

        // update the associated address in the auction config
        if domain == associated_orbital_auction.src_domain {
            associated_orbital_auction.src_clearing_acc_addr = Some(clearing_account_config.addr);
        } else if domain == associated_orbital_auction.dest_domain {
            associated_orbital_auction.dest_clearing_acc_addr = Some(clearing_account_config.addr);
        }
        ORBITAL_AUCTIONS.save(deps.storage, auction_id, &associated_orbital_auction)?;

        // if both clearing accounts are prepared, we can instantiate the auction
        if associated_orbital_auction.prepared_clearing_accounts() {
            admin::try_instantiate_auction(deps, env, string_ica_identifier, auction_id)?;
        }
    }

    Ok(Response::default())
}
