#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure, to_json_binary, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, IbcMsg,
    IbcTimeout, MessageInfo, Response, StdResult, Timestamp, Uint128,
};

use cw2::set_contract_version;
use cw_utils::{must_pay, Duration, Expiration};
use orbital_utils::intent::Intent;

use crate::{
    error::ContractError,
    helpers::{add_intent, get_bid},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{next_id, ACTIVE_AUCTION, CONFIG, IDS, INTENTS, QUEUE},
    types::{ActiveAuction, Config, NewAuction},
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
        ExecuteMsg::NewIntent(new_intent) => execute_new_intent(deps, env, info, new_intent),
        ExecuteMsg::AuctionTick {} => execute_auction_tick(deps, env, info),
        ExecuteMsg::AuctionBid { bidder } => execute_auction_bid(deps, env, info, bidder),
        ExecuteMsg::AuctionClaim {} => execute_auction_claim(deps, env, info),
    }
}

pub fn execute_new_intent(
    deps: DepsMut,
    env: Env,
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

pub fn execute_auction_tick(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut auction = ACTIVE_AUCTION.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    // ensure the auction is expired
    ensure!(
        auction.end_time.is_expired(&env.block),
        ContractError::AuctionExpired
    );

    // get next intent
    let Some(last_id) = QUEUE.dequeue(deps.storage)? else {
        return Err(ContractError::QueueIsEmpty);
    };
    let next_id = last_id + 1;

    let next_intent = INTENTS
        .load(deps.storage, next_id)
        .map_err(|_| ContractError::IntentNotFound)?;

    // TODO: Set our "watcher" to wait for the MM to proof he deposited the funds on the intent addr
    // TODO: Before we add it to the watcher, make sure that there is a bidder, else do nothing
    // ensure!(
    //     auction.bidder.is_some(),
    //     ContractError::NoBids
    // );

    // set the active auction to the next intent
    ACTIVE_AUCTION.save(
        deps.storage,
        &ActiveAuction {
            end_time: config.duration.after(&env.block),
            highest_bid: next_intent.ask_coin.amount,
            bidder: None,
            intent_id: next_id,
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
    let bid = get_bid(deps, &config, &env, &info)?;

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

pub fn execute_auction_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // We need to get the proof of the balance
    // verify the proof is legit
    // send the account controller a message telling him that this intent was fulfilled.
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAuction {} => {
            let auction = ACTIVE_AUCTION.load(deps.storage)?;

            if auction.end_time.is_expired(&env.block) {
                to_json_binary(&None::<()>)
            } else {
                to_json_binary(&Some(auction))
            }
        }
    }
}
