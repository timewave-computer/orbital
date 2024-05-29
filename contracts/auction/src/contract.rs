#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};

use cw2::set_contract_version;
use orbital_utils::intent::Intent;

use crate::{
    error::ContractError,
    helpers::{add_intent, get_bid, remove_intent},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{ACTIVE_AUCTION, CONFIG, IDS, INTENTS},
    types::{ActiveAuction, Config, TestAccountExecuteMsg},
};

const CONTRACT_NAME: &str = "crates.io:vesting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        account_addr: deps.api.addr_validate(&msg.account_addr)?,
        bond_amount: msg.bond_amount,
        increment_decimal: Decimal::bps(msg.increment_bps),
        duration: msg.duration,
        fulfillment_timeout: msg.fulfillment_timeout,
    };

    CONFIG.save(deps.storage, &config)?;
    IDS.save(deps.storage, &0)?;

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
        ExecuteMsg::NewIntent(new_intent) => execute_new_intent(deps, info, new_intent),
        ExecuteMsg::AuctionTick {} => execute_auction_tick(deps, env),
        ExecuteMsg::AuctionBid { bidder } => execute_auction_bid(deps, env, info, bidder),
    }
}

pub fn execute_new_intent(
    deps: DepsMut,
    info: MessageInfo,
    new_intent: Intent,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the account (controller/account wrapper) can create add new intents
    ensure!(
        info.sender == config.account_addr,
        ContractError::Unauthorized("Only account can create new auctions".to_string())
    );

    // add the intent to our system
    add_intent(deps, new_intent)?;

    Ok(Response::new())
}

pub fn execute_auction_tick(mut deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let auction = ACTIVE_AUCTION.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let curr_intent = INTENTS.load(deps.storage, auction.intent_id)?;

    // ensure the auction is expired
    ensure!(
        auction.end_time.is_expired(&env.block),
        ContractError::AuctionExpired
    );

    // If we have a bidder addr, meaning we have a winner, so send a msg to the account controller
    if auction.bidder.is_some() {
        // Send msg to the account controller, and tell him the auction is finished, and
        let msg = WasmMsg::Execute {
            contract_addr: config.account_addr.to_string(),
            msg: to_json_binary(&TestAccountExecuteMsg::AuctionFinished {
                original_intent: curr_intent,
                id: auction.intent_id,
                winning_bid: auction.highest_bid,
                bidder: auction.bidder.clone(),
            })?,
            funds: vec![],
        };

        return Ok(Response::default().add_message(msg));
    }

    let Some(next_intent_id) = remove_intent(deps.branch())? else {
        // If we don't have an id here, itm eans we have nothing in the queue
        return Ok(Response::default());
    };

    let next_intent = INTENTS
        .load(deps.storage, next_intent_id)
        .map_err(|_| ContractError::IntentNotFound)?;

    // set the active auction to the next intent
    ACTIVE_AUCTION.save(
        deps.storage,
        &ActiveAuction {
            end_time: config.duration.after(&env.block),
            highest_bid: next_intent.ask_coin.amount,
            bidder: None,
            intent_id: next_intent_id,
        },
    )?;

    Ok(Response::default())
}

pub fn execute_auction_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bidder: String,
) -> Result<Response, ContractError> {
    let mut auction = ACTIVE_AUCTION.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    // ensure the auction is still active
    ensure!(
        !auction.end_time.is_expired(&env.block),
        ContractError::AuctionExpired
    );

    // check for the bid amounts, that it included the bond and what is the amount he bid
    let bid = get_bid(&config, &info)?;

    // make sure the bid is at least above the othere bid by the increment
    let min_bid = auction.highest_bid.checked_add(
        Decimal::from_atomics(auction.highest_bid, 0)?
            .checked_mul(config.increment_decimal)?
            .to_uint_floor(),
    )?;
    // ensure the bid is higher then the previous bid plus the increment
    ensure!(bid.amount >= min_bid, ContractError::BidTooLow(min_bid));

    // if everything checks out then set it as the next winner
    auction.highest_bid = bid.amount;
    auction.bidder = Some(bidder);

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAuction {} => to_json_binary(&ACTIVE_AUCTION.load(deps.storage)?),
    }
}
