#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure, to_json_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, Storage, Uint128, Uint64,
};
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

use crate::{
    admin,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    solver,
    state::{
        AuctionConfig, AuctionPhase, AuctionPhaseConfig, AuctionRound, Batch, Bid,
        RoundPhaseExpirations, UserIntent, ACTIVE_AUCTION, AUCTION_ARCHIVE, AUCTION_CONFIG,
        ORBITAL_CORE, ORDERBOOK, POSTED_BONDS,
    },
};

pub const CONTRACT_NAME: &str = "orbital-auction";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type QueryDeps<'a> = Deps<'a, NeutronQuery>;
pub type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;

#[entry_point]
pub fn instantiate(
    deps: ExecuteDeps,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let auction_config = AuctionConfig {
        batch_size: msg.batch_size,
        auction_phases: AuctionPhaseConfig {
            auction_duration: msg.auction_duration,
            filling_window_duration: msg.filling_window_duration,
            cleanup_window_duration: msg.cleanup_window_duration,
        },
        route_config: msg.route_config,
        solver_bond: msg.solver_bond,
    };

    let active_round = AuctionRound {
        id: Uint64::zero(),
        // start the auction cycle with an empty batch
        batch: Batch {
            batch_capacity: msg.batch_size,
            batch_size: Uint128::zero(),
            user_intents: vec![],
            current_bid: None,
        },
        phases: RoundPhaseExpirations::from_auction_config(
            &auction_config.auction_phases,
            &env.block,
        )?,
    };

    // set the sender as orbital-core
    ORBITAL_CORE.save(deps.storage, &info.sender)?;

    // save the auction-related configs
    AUCTION_CONFIG.save(deps.storage, &auction_config)?;
    ACTIVE_AUCTION.save(deps.storage, &active_round)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: ExecuteDeps,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        // permisionless action
        ExecuteMsg::Tick { mock_fill_status } => try_finalize_round(deps, env, mock_fill_status),

        // solver actions
        ExecuteMsg::Bid { amount } => solver::try_bid(
            deps,
            Bid {
                solver: info.sender,
                amount,
                bid_block: env.block,
            },
        ),
        ExecuteMsg::PostBond {} => solver::try_post_bond(deps, info),
        ExecuteMsg::WithdrawBond {} => solver::try_withdraw_posted_bond(deps, info),

        // admin-gated actions. should we add a RemoveOrder?
        // if order is not included in a batch yet, seems like there is no risk to that.
        ExecuteMsg::AddOrder(user_intent) => admin::enqueue_user_intent(deps, info, user_intent),
        ExecuteMsg::Pause {} => unimplemented!(),
        ExecuteMsg::Resume {} => unimplemented!(),
    }
}

/// action to finalize the current round and prepare for the next one.
// `mock_fill_status` here is temp for unit testing
fn try_finalize_round(
    deps: ExecuteDeps,
    env: Env,
    mock_fill_status: bool,
) -> NeutronResult<Response<NeutronMsg>> {
    let active_auction = ACTIVE_AUCTION.load(deps.storage)?;

    // first we check if the active auction is not yet started. this can be the case
    // if (and only if) it's been finalized already and is due to start once it
    // enters its bidding phase.
    ensure!(
        active_auction
            .phases
            .start_expiration
            .is_expired(&env.block),
        ContractError::AuctionPhaseError {}
    );

    // depending on the phase we are in, finalization is handled differently:
    match query_active_auction(deps.as_ref(), env)? {
        // no-op as there is nothing to finalize in the bidding phase
        AuctionPhase::Bidding => Err(ContractError::AuctionPhaseError {}.into()),
        AuctionPhase::Filling => {
            // if no bids are placed, we finalize the round and prepare for the next one
            if active_auction.batch.current_bid.is_none() {
                start_new_round(deps.storage, active_auction)?;
                process_orderbook(deps.storage)?;
                return Ok(Response::default());
            }

            let order_filled = query_order_filling_status(deps.as_ref(), mock_fill_status)?;

            // if order is filled, we finalize the round and prepare for the next one.
            if order_filled {
                start_new_round(deps.storage, active_auction)?;
                process_orderbook(deps.storage)?;
                // TODO: credit the users & solver
                Ok(Response::default())
            } else {
                // if order is not yet filled, we do nothing.
                // solver still has time to fill their order and finalize the round.
                Ok(Response::default())
            }
        }
        // in the cleanup phase, we finalize the round and prepare for the next one.
        AuctionPhase::Cleanup => {
            // if no bids are placed, we finalize the round and prepare for the next one
            if active_auction.batch.current_bid.is_none() {
                start_new_round(deps.storage, active_auction)?;
                process_orderbook(deps.storage)?;
                return Ok(Response::default());
            }

            let order_filled = query_order_filling_status(deps.as_ref(), mock_fill_status)?;

            // if the solver succeeded, we do not slash the solver and prepare for the next round.
            if order_filled {
                start_new_round(deps.storage, active_auction)?;
                process_orderbook(deps.storage)?;
                // TODO: credit the users & solver
                Ok(Response::default())
            } else {
                // if the solver failed to fill the order, we slash the solver, refund the users,
                // and prepare for the next round.
                let winning_bid = active_auction.batch.current_bid.clone().unwrap();
                slash_solver(deps.storage, winning_bid.solver)?;

                // then start the new round
                start_new_round(deps.storage, active_auction)?;
                process_orderbook(deps.storage)?;

                // TODO: refund users
                Ok(Response::default())
            }
        }
        // if we are out of sync, that means the round had failed to be finalized during
        // the filling and cleanup phases. given that this should only happen in case of
        // any sort of infra failure, this likely involves pausing the auction
        // and requiring admin intervention.
        AuctionPhase::OutOfSync => unimplemented!(),
    }
}

/// moves the user intents from the orderbook queue to the active auction batch
fn process_orderbook(storage: &mut dyn Storage) -> StdResult<()> {
    let mut active_auction = ACTIVE_AUCTION.load(storage)?;
    let mut batch_capacity = active_auction.batch.remaining_capacity();

    // we iterate over the head of the orderbook while the batch is not full
    while !ORDERBOOK.is_empty(storage)? && batch_capacity > Uint128::zero() {
        // grab the head of the orderbook
        if let Some(intent) = ORDERBOOK.pop_front(storage)? {
            if intent.amount <= batch_capacity {
                // if it fits, we add it to the batch and update the capacity
                println!("Moved head of orderbook into the batch: {:?}", intent);
                active_auction.batch.user_intents.push(intent.clone());
                batch_capacity = batch_capacity.checked_sub(intent.amount)?;
            } else {
                // if it doesn't fit, we push it back to the orderbook and break the loop
                println!("Head of orderbook doesn't fit in the batch!");
                ORDERBOOK.push_front(storage, &intent)?;
                break;
            }
        } else {
            println!("Fully processed the orderbook!");
        }
    }

    ACTIVE_AUCTION.save(storage, &active_auction)?;

    Ok(())
}

/// queries the auction config to find the configured bond for this auction
/// and slashes the solver's bond accordingly.
fn slash_solver(storage: &mut dyn Storage, solver: Addr) -> StdResult<()> {
    let auction_config = AUCTION_CONFIG.load(storage)?;
    let mut posted_bond = POSTED_BONDS.load(storage, solver.to_string())?;
    posted_bond.amount = posted_bond
        .amount
        .checked_sub(auction_config.solver_bond.amount)?;

    POSTED_BONDS.save(storage, solver.to_string(), &posted_bond)?;

    Ok(())
}

/// archives the current round and prepares the next active round
fn start_new_round(storage: &mut dyn Storage, active_auction: AuctionRound) -> StdResult<()> {
    // first initialize the next round and replace the active round with it
    let next_round = active_auction.advance(storage)?;
    ACTIVE_AUCTION.save(storage, &next_round)?;

    // then archive the current round
    AUCTION_ARCHIVE.push_front(storage, &active_auction)?;

    Ok(())
}

/// query orbital-core contract to check if solver had deposited the funds
/// into the clearing account
fn query_order_filling_status(deps: QueryDeps, mock_fill_status: bool) -> NeutronResult<bool> {
    let _orbital_core = ORBITAL_CORE.load(deps.storage)?;

    // TODO: query orbital-core
    Ok(mock_fill_status)
}

#[entry_point]
pub fn query(deps: QueryDeps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&ORBITAL_CORE.load(deps.storage)?),
        QueryMsg::AuctionConfig {} => to_json_binary(&AUCTION_CONFIG.load(deps.storage)?),
        QueryMsg::Orderbook { from, limit } => to_json_binary(&query_orderbook(deps, from, limit)?),
        QueryMsg::PostedBond { solver } => to_json_binary(&query_posted_bond(deps, solver)?),
        QueryMsg::ActiveRound {} => to_json_binary(&ACTIVE_AUCTION.load(deps.storage)?),
        QueryMsg::AuctionPhase {} => to_json_binary(&query_active_auction(deps, env)?),
    }
}

fn query_active_auction(deps: QueryDeps, env: Env) -> StdResult<AuctionPhase> {
    let active_round_config = ACTIVE_AUCTION.load(deps.storage)?;
    let phase = active_round_config.phases.get_current_phase(&env.block);

    Ok(phase)
}

fn query_posted_bond(deps: QueryDeps, solver: String) -> StdResult<Coin> {
    let auction_config = AUCTION_CONFIG.load(deps.storage)?;
    let posted_bond = POSTED_BONDS
        .may_load(deps.storage, solver)?
        .unwrap_or(coin(0, auction_config.solver_bond.denom));

    Ok(posted_bond)
}

fn query_orderbook(
    deps: QueryDeps,
    from: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Vec<UserIntent>> {
    // query the first `limit` elements starting from `from` in the ORDERBOOK storage Dequeue item
    let orderbook_size = ORDERBOOK.len(deps.storage)?;
    let from = from.unwrap_or(0);
    let limit = limit.unwrap_or(orderbook_size);

    let resp: Vec<UserIntent> = ORDERBOOK
        .iter(deps.storage)?
        .skip(from as usize)
        .take(limit as usize)
        .map(|r| r.unwrap()) // TODO: clean this up
        .collect();

    Ok(resp)
}

#[entry_point]
pub fn reply(_deps: ExecuteDeps, _env: Env, _msg: Reply) -> StdResult<Response<NeutronMsg>> {
    unimplemented!()
}

#[entry_point]
pub fn migrate(_deps: ExecuteDeps, _env: Env, _msg: MigrateMsg) -> StdResult<Response<NeutronMsg>> {
    unimplemented!()
}
