use std::{collections::HashMap, str::FromStr};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint64, WasmMsg
};

use auction::msg::ExecuteMsg as AuctionExecuteMsg;
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronError, NeutronResult,
};
use orbital_utils::domain::OrbitalDomain;
use polytone::callbacks::CallbackRequest;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    polytone_helpers::{
        get_note_execute_neutron_msg, get_proxy_query_balances_message,
        query_polytone_proxy_address, try_handle_callback, REGISTER_DOMAIN_CALLBACK_ID,
    },
    state::{ADMIN, AUCTION_ADDR, DOMAIN_TO_NOTE, LEDGER, NOTE_TO_DOMAIN, USER_DOMAINS},
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
        ExecuteMsg::NewIntent(new_intent) => {
            // send new intent to the auction addr
            let auction_addr = AUCTION_ADDR.load(deps.storage)?;

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
                msg: to_json_binary(&AuctionExecuteMsg::NewIntent(new_intent))?,
                funds: vec![],
            };

            Ok(Response::new().add_message(msg))
        }
        ExecuteMsg::VerifyAuction {
            original_intent,
            winning_bid,
            bidder,
        } => {
            // Verify the sender is the auction address
            let auction_addr = AUCTION_ADDR.load(deps.storage)?;

            ensure!(
                auction_addr == info.sender,
                StdError::generic_err(
                    "sender is not the auction addr",
                )
            );
            
            // TODO: verify the MM deposited the funds into the account he was supposed to
            // update ledger to reflect the change and unlock funds to the MM

            // if MM didn't fulfill, send a slash msg to the auction addr
            Ok(Response::new())
        }
    }
}

pub fn try_sync_domain(
    deps: ExecuteDeps,
    env: Env,
    domain: OrbitalDomain,
) -> NeutronResult<Response<NeutronMsg>> {
    let note_addr = DOMAIN_TO_NOTE.load(deps.storage, domain.value())?;
    let proxy_addr = USER_DOMAINS.load(deps.storage, domain.value())?;

    let proxy_query_balances_msg =
        get_proxy_query_balances_message(env.clone(), proxy_addr, note_addr.to_string())?;

    let polytone_init_msg = get_note_execute_neutron_msg(
        vec![proxy_query_balances_msg.into()],
        Uint64::new(120),
        note_addr,
        Some(CallbackRequest {
            receiver: env.contract.address.to_string(),
            msg: to_json_binary(&REGISTER_DOMAIN_CALLBACK_ID)?,
        }),
    )?;

    Ok(Response::new().add_message(polytone_init_msg))
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

            let mut ledger_result = String::new();
            for ledger in ledgers {
                let result_entry = format!("{} : {}", ledger.0, ledger.1);
                ledger_result = format!("{}\n{}", ledger_result, result_entry);
            }
            to_json_binary(&ledger_result)
        }
    }
}
