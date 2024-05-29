use std::{collections::HashMap, str::FromStr};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, BankQuery, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, Uint64, WasmMsg,
};

use auction::msg::ExecuteMsg as AuctionExecuteMsg;
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronError, NeutronResult,
};
use orbital_utils::{domain::OrbitalDomain, intent::Intent};
use polytone::callbacks::CallbackRequest;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    polytone_helpers::{
        get_note_execute_neutron_msg, get_note_query_neutron_msg, query_polytone_proxy_address,
        try_handle_callback, REGISTER_DOMAIN_CALLBACK_ID, SYNC_DOMAIN_CALLBACK_ID,
    },
    state::{ADMIN, AUCTION_ADDR, DOMAIN_TO_NOTE, LEDGER, NOTE_TO_DOMAIN, USER_DOMAINS},
    types::QueryRecievedFundsOnDestDomain,
};

const CONTRACT_NAME: &str = "crates.io:account";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;
type QueryDeps<'a> = Deps<'a, NeutronQuery>;
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: ExecuteDeps,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ADMIN.save(deps.storage, &info.sender)?;

    // we initialize an empty ledger for the user to enable fund deposits/withdrawals
    LEDGER.save(
        deps.storage,
        OrbitalDomain::Neutron.value(),
        &HashMap::new(),
    )?;

    // root domain address is this contract
    USER_DOMAINS.save(
        deps.storage,
        OrbitalDomain::Neutron.value(),
        &env.contract.address.to_string(),
    )?;

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
        ExecuteMsg::RegisterDomain { domain, note_addr } => {
            execute_register_domain(deps, env, info, domain, note_addr)
        }
        ExecuteMsg::Callback(callback_msg) => try_handle_callback(env, deps, info, callback_msg),
        ExecuteMsg::Sync { domain } => try_sync_domain(deps, env, domain),
        ExecuteMsg::UpdateAuctionAddr { auction_addr } => {
            AUCTION_ADDR.save(deps.storage, &deps.api.addr_validate(&auction_addr)?)?;
            Ok(Response::new())
        }
        ExecuteMsg::NewIntent(new_intent) => execute_new_intent(deps, env, info, new_intent),
        ExecuteMsg::VerifyAuction {
            original_intent,
            winning_bid,
            bidder,
            mm_addr,
        } => {
            // Verify the sender is the auction address
            let auction_addr = AUCTION_ADDR.load(deps.storage)?;

            ensure!(
                auction_addr == info.sender,
                StdError::generic_err("sender is not the auction addr",)
            );

            let note = DOMAIN_TO_NOTE.load(deps.storage, original_intent.ask_domain.value())?;

            // Query MM deposit address over polytone
            let polytone_query_msg = get_note_query_neutron_msg(
                vec![QueryRequest::Bank(BankQuery::Balance {
                    address: original_intent.deposit_addr.clone(),
                    denom: original_intent.ask_coin.denom.clone(),
                })],
                Uint64::new(120),
                note,
                CallbackRequest {
                    receiver: env.contract.address.to_string(),
                    msg: to_json_binary(&QueryRecievedFundsOnDestDomain {
                        intent: original_intent,
                        winning_bid,
                        bidder,
                        mm_addr,
                    })?,
                },
            )?;

            // if MM didn't fulfill, send a slash msg to the auction addr
            Ok(Response::new().add_message(polytone_query_msg))
        }
        ExecuteMsg::WithdrawFunds { domain, coin, dest } => {
            execute_withdraw_funds(deps, env, info, domain, coin, dest)
        }
    }
}

pub fn execute_new_intent(
    deps: ExecuteDeps,
    _env: Env,
    _info: MessageInfo,
    new_intent: Intent,
) -> NeutronResult<Response<NeutronMsg>> {
    // send new intent to the auction addr
    let auction_addr = AUCTION_ADDR.load(deps.storage)?;

    let ask_demain_addr = USER_DOMAINS.load(deps.storage, new_intent.ask_domain.value())?;

    // Verify the funds are in the senders ledger
    let ledger = LEDGER.load(deps.storage, new_intent.offer_domain.value())?;
    let balance = *ledger.get(new_intent.offer_coin.denom.as_str()).unwrap();

    if balance < new_intent.offer_coin.amount.u128() {
        return Err(NeutronError::Std(StdError::generic_err(
            "Insufficient funds",
        )));
    }

    // send message to add the intent to the queue
    let msg = WasmMsg::Execute {
        contract_addr: auction_addr.to_string(),
        msg: to_json_binary(&AuctionExecuteMsg::NewIntent(new_intent, ask_demain_addr))?,
        funds: vec![],
    };

    Ok(Response::new().add_message(msg))
}

pub fn execute_withdraw_funds(
    _deps: ExecuteDeps,
    _env: Env,
    _info: MessageInfo,
    _domain: OrbitalDomain,
    _coin: Coin,
    _dest: String,
) -> NeutronResult<Response<NeutronMsg>> {
    Ok(Response::default())
}

pub fn try_sync_domain(
    deps: ExecuteDeps,
    env: Env,
    domain: OrbitalDomain,
) -> NeutronResult<Response<NeutronMsg>> {
    let note_addr = DOMAIN_TO_NOTE.load(deps.storage, domain.value())?;
    let proxy_addr = USER_DOMAINS.load(deps.storage, domain.value())?;

    let query_request: QueryRequest<Empty> = cosmwasm_std::BankQuery::AllBalances {
        address: proxy_addr.to_string(),
    }
    .into();

    let polytone_sync_balance_msg = get_note_query_neutron_msg(
        vec![query_request],
        Uint64::new(120),
        note_addr,
        CallbackRequest {
            receiver: env.contract.address.to_string(),
            msg: to_json_binary(&SYNC_DOMAIN_CALLBACK_ID)?,
        },
    )?;

    Ok(Response::new().add_message(polytone_sync_balance_msg))
}

pub fn execute_register_domain(
    deps: ExecuteDeps,
    env: Env,
    _info: MessageInfo,
    domain: OrbitalDomain,
    note_addr: String,
) -> NeutronResult<Response<NeutronMsg>> {
    // validate the note address (on neutron chain)
    let note = deps.api.addr_validate(&note_addr)?;
    NOTE_TO_DOMAIN.save(deps.storage, note.clone(), &domain)?;
    DOMAIN_TO_NOTE.save(deps.storage, domain.value(), &note)?;

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
pub fn query(deps: QueryDeps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryDomainAddr { domain } => {
            let domain = OrbitalDomain::from_str(domain.as_str())?;
            let user_remote_addr = USER_DOMAINS.load(deps.storage, domain.value())?;
            to_json_binary(&user_remote_addr)
        }
        QueryMsg::QueryAllDomains {} => {
            let all_domains =
                USER_DOMAINS.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);
            let mut domain_result = String::new();
            for domain in all_domains {
                let entry = domain?;
                let result_entry = format!("{} : {}", entry.0, entry.1);
                domain_result = format!("{}\n{}", domain_result, result_entry);
            }
            to_json_binary(&domain_result)
        }
        QueryMsg::QueryProxyAccount { domain } => {
            let domain = OrbitalDomain::from_str(domain.as_str())?;
            let note_addr = DOMAIN_TO_NOTE.load(deps.storage, domain.value())?;
            let proxy_address = query_polytone_proxy_address(
                env.contract.address.to_string(),
                note_addr.to_string(),
                deps.querier,
            )?;
            to_json_binary(&proxy_address)
        }
        QueryMsg::QueryLedger { domain } => {
            let domain = OrbitalDomain::from_str(domain.as_str())?;

            let ledgers = LEDGER.load(deps.storage, domain.value())?;

            let mut ledger_result = vec![];
            for (denom, bal) in ledgers {
                ledger_result.push((denom, bal));
            }
            to_json_binary(&ledger_result)
        }
        QueryMsg::QueryAllLedgers {} => {
            let all_ledgers =
                LEDGER.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);
            let mut ledger_results = vec![];

            for ledger in all_ledgers {
                let (domain, balances) = ledger?;
                for (denom, bal) in balances {
                    ledger_results.push((domain.to_string(), denom, bal));
                }
            }
            to_json_binary(&ledger_results)
        }
    }
}
