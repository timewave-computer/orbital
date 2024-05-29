#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};

use cw2::set_contract_version;
use cw_utils::must_pay;
use orbital_utils::intent::Intent;

use crate::{
    error::ContractError,
    helpers::{add_intent, next_intent},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{ACTIVE_AUCTION, BONDS, CONFIG, IDS, INTENTS, TO_VERIFY},
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
        bond: msg.bond,
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
        ExecuteMsg::NewIntent(new_intent, deposit_addr) => execute_new_intent(deps, info, new_intent, deposit_addr),
        ExecuteMsg::AuctionTick {} => execute_auction_tick(deps, env),
        ExecuteMsg::AuctionBid { bidder } => execute_auction_bid(deps, env, info, bidder),
        ExecuteMsg::Bond {} => {
            let config = CONFIG.load(deps.storage)?;
            let amount = must_pay(&info, &config.bond.denom)?;
            ensure!(
                amount == config.bond.amount,
                ContractError::BondMismatch(config.bond)
            );
            BONDS.save(deps.storage, info.sender, &info.funds[0])?;
            Ok(Response::new())
        }
        ExecuteMsg::Slash {} => {
            ensure!(
                info.sender == CONFIG.load(deps.storage)?.account_addr,
                ContractError::Unauthorized("Only account can slash".to_string())
            );
            BONDS.load(deps.storage, info.sender.clone())?;
            BONDS.remove(deps.storage, info.sender);
            Ok(Response::new())
        }
        ExecuteMsg::Fulfilled { id } => todo!(),
    }
}

pub fn execute_new_intent(
    deps: DepsMut,
    info: MessageInfo,
    new_intent: Intent,
    deposit_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the account (controller/account wrapper) can create add new intents
    ensure!(
        info.sender == config.account_addr,
        ContractError::Unauthorized("Only account can create new auctions".to_string())
    );

    // add the intent to our system
    add_intent(deps, new_intent.into_saved_intent(deposit_addr))?;

    Ok(Response::new())
}

pub fn execute_auction_tick(mut deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut msgs: Vec<WasmMsg> = Vec::with_capacity(1);

    let curr_auction = ACTIVE_AUCTION.load(deps.storage)?;

    // ensure the auction is expired
    ensure!(
        curr_auction.end_time.is_expired(&env.block),
        ContractError::AuctionNotExpired
    );

    let Some(next_intent_id) = next_intent(deps.branch())? else {
        // If we don't have an id here, it means we have nothing in the queue
        return Ok(Response::default());
    };

    // If ive got something to verify, i verify it.
    match TO_VERIFY.load(deps.storage) {
        Ok(auction) => {
            // if we have an auction to verify, we need to do that first
            let mut intent = INTENTS.load(deps.storage, auction.intent_id)?;

            if !intent.is_verified {
                msgs.push(WasmMsg::Execute {
                    contract_addr: config.account_addr.to_string(),
                    msg: to_json_binary(&TestAccountExecuteMsg::VerifyAuction {
                        original_intent: intent.clone(),
                        winning_bid: auction.highest_bid.amount,
                        bidder: auction.bidder.unwrap(),
                    })?,
                    funds: vec![],
                });
                intent.is_verified = true;
                INTENTS.save(deps.storage, auction.intent_id, &intent)?;
                TO_VERIFY.remove(deps.storage);
            }
        }
        Err(_) => (),
    };

    if curr_auction.bidder.is_some() {
        //we havea bidder, so add it to the verify storage
        TO_VERIFY.save(deps.storage, &curr_auction)?;
    }

    let next_intent = INTENTS
        .load(deps.storage, next_intent_id)
        .map_err(|_| ContractError::IntentNotFound)?;

    // set the active auction to the next intent
    ACTIVE_AUCTION.save(
        deps.storage,
        &ActiveAuction {
            end_time: config.duration.after(&env.block),
            highest_bid: next_intent.ask_coin,
            bidder: None,
            intent_id: next_intent_id,
            verified: false,
        },
    )?;

    Ok(Response::default().add_messages(msgs))
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
        ContractError::AuctionNotExpired
    );
    // make sure the sender has bond to bid
    ensure!(
        BONDS.has(deps.storage, info.sender.clone()),
        ContractError::NoBond
    );

    // check for the bid amounts
    let bid = must_pay(&info, &auction.highest_bid.denom)?;

    // make sure the bid is at least above the othere bid by the increment
    let min_bid = auction.highest_bid.amount.checked_add(
        Decimal::from_atomics(auction.highest_bid.amount, 0)?
            .checked_mul(config.increment_decimal)?
            .to_uint_floor(),
    )?;
    // ensure the bid is higher then the previous bid plus the increment
    ensure!(bid >= min_bid, ContractError::BidTooLow(min_bid));

    // if everything checks out then set it as the next winner
    auction.highest_bid.amount = bid;
    auction.bidder = Some(bidder);

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAuction {} => to_json_binary(&ACTIVE_AUCTION.load(deps.storage)?),
    }
}
