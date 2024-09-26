use crate::{
    admin_logic::admin,
    error::ContractError,
    icq::{self},
    state::{ClearingAccountConfig, OrbitalDomainConfig, UserConfig, USER_NONCE},
    user_logic::user,
    utils::{
        extract_ica_identifier_from_port, fees::flatten_ibc_fees_amt, get_ica_identifier,
        OpenAckVersion,
    },
};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{CLEARING_ACCOUNTS, ORBITAL_DOMAINS, USER_CONFIGS},
};
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, Uint64,
};
use cw2::set_contract_version;
use cw_ownable::{get_ownership, initialize_owner};
use cw_utils::must_pay;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery, types::ProtobufAny},
    interchain_queries::v047::queries::{query_balance, BalanceResponse},
    query::min_ibc_fee::query_min_ibc_fee,
    sudo::msg::SudoMsg,
    NeutronResult,
};
use prost::Message;

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
        ExecuteMsg::RegisterBalancesQuery {
            connection_id,
            update_period,
            addr,
            denoms,
        } => icq::register_balances_query(connection_id, addr, denoms, update_period),
        ExecuteMsg::UserWithdrawFunds { domain, coin, dest } => {
            try_withdraw_from_remote_domain(deps, info, domain, coin, dest)
        }
    }
}

fn try_withdraw_from_remote_domain(
    deps: ExecuteDeps,
    info: MessageInfo,
    domain: String,
    coin: Coin,
    dest: String,
) -> NeutronResult<Response<NeutronMsg>> {
    let user_config = USER_CONFIGS.load(deps.storage, info.sender.to_string())?;

    // first we validate that user is registered to the domain from which they
    // want to withdraw funds from
    ensure!(
        user_config.registered_domains.contains(&domain),
        ContractError::UserNotRegisteredToDomain(domain)
    );

    // validate that IBC fees are covered by the caller
    let min_ibc_fee = query_min_ibc_fee(deps.as_ref())?;
    let total_fee_amt = flatten_ibc_fees_amt(&min_ibc_fee.min_fee);
    let paid_amt = must_pay(&info, "untrn").map_err(ContractError::FeePaymentError)?;

    ensure!(
        paid_amt >= total_fee_amt,
        ContractError::Std(StdError::generic_err("insufficient fee coverage"))
    );

    // derive the port associated with user's clearing account
    let ica_identifier = get_ica_identifier(user_config.id, domain.to_string());

    let user_clearing_acc_config = CLEARING_ACCOUNTS
        .load(deps.storage, ica_identifier.to_string())?
        .ok_or_else(|| ContractError::UserNotRegisteredToDomain(domain))?;

    // generate the transfer message to be executed on target domain
    let proto_coin = ProtoCoin {
        denom: coin.denom,
        amount: coin.amount.to_string(),
    };
    let bank_msg = cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend {
        from_address: user_clearing_acc_config.addr,
        to_address: dest,
        amount: vec![proto_coin],
    };

    let proto_msg = generate_proto_msg(bank_msg, "/cosmos.bank.v1beta1.MsgSend")?;

    let withdraw_tx: NeutronMsg = NeutronMsg::submit_tx(
        user_clearing_acc_config.controller_connection_id,
        ica_identifier,
        vec![proto_msg],
        "".to_string(),
        60,
        min_ibc_fee.min_fee,
    );

    Ok(Response::default().add_message(withdraw_tx))
}

fn generate_proto_msg(msg: impl Message, type_url: &str) -> NeutronResult<ProtobufAny> {
    let buf = msg.encode_to_vec();

    let any_msg = ProtobufAny {
        type_url: type_url.to_string(),
        value: Binary::from(buf),
    };
    Ok(any_msg)
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
    }
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
