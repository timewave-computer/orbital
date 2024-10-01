#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, Uint64,
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
        AuctionConfig, AuctionPhase, AuctionPhaseConfig, AuctionRound, BatchStatus,
        RoundPhaseExpirations, UserIntent, ACTIVE_AUCTION_CONFIG, ADMIN, AUCTION_CONFIG, ORDERBOOK,
        POSTED_BONDS,
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

    let active_round_config = AuctionRound {
        id: Uint64::zero(),
        // start the auction cycle with an empty batch
        batch: BatchStatus::Empty {},
        phases: RoundPhaseExpirations::from_auction_config(
            &auction_config.auction_phases,
            &env.block,
        )?,
    };

    // set the sender as admin
    ADMIN.save(deps.storage, &info.sender)?;

    // save the auction-related configs
    AUCTION_CONFIG.save(deps.storage, &auction_config)?;
    ACTIVE_AUCTION_CONFIG.save(deps.storage, &active_round_config)?;

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
        ExecuteMsg::FinalizeRound {} => try_finalize_round(deps, env),

        // solver actions
        ExecuteMsg::Bid {} => unimplemented!(),
        ExecuteMsg::PostBond {} => solver::try_post_bond(deps, info),
        ExecuteMsg::WithdrawBond {} => solver::try_withdraw_posted_bond(deps, info),
        ExecuteMsg::Prove {} => unimplemented!(),

        // admin-gated actions. should we add a RemoveOrder?
        // if order is not included in a batch yet, seems like there is no risk to that.
        ExecuteMsg::AddOrder(user_intent) => admin::enqueue_user_intent(deps, info, user_intent),
        ExecuteMsg::Pause {} => unimplemented!(),
        ExecuteMsg::Resume {} => unimplemented!(),
    }
}

/// action to finalize the current round and prepare for the next one.
fn try_finalize_round(deps: ExecuteDeps, env: Env) -> NeutronResult<Response<NeutronMsg>> {
    let active_auction = ACTIVE_AUCTION_CONFIG.load(deps.storage)?;

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
    match query_active_auction_phase(deps.as_ref(), env)? {
        // no-op as there is nothing to finalize in the bidding phase
        AuctionPhase::Bidding => Err(ContractError::AuctionPhaseError {}.into()),
        // in the filling phase, if we confirm that the solver had filled the order correctly,
        // we finalize the round and prepare for the next one.
        // if the solver failed to fill the order, we wait for the cleanup phase to finalize.
        // no slashing happens in this phase.
        AuctionPhase::Filling => unimplemented!(),
        // in the cleanup phase, we finalize the round and prepare for the next one.
        // if the solver failed to fill the order, we slash the solver, refund the users,
        // and prepare for the next round.
        // if the solver succeeded, we do not slash the solver and prepare for the next round.
        AuctionPhase::Cleanup => unimplemented!(),
        // if we are out of sync, that means the round had failed to be finalized during
        // the filling and cleanup phases. given that this should only happen in case of
        // any sort of infra failure, this likely involves pausing the auction
        // and requiring admin intervention.
        AuctionPhase::OutOfSync => unimplemented!(),
    }
}

#[entry_point]
pub fn query(deps: QueryDeps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&ADMIN.load(deps.storage)?),
        QueryMsg::AuctionConfig {} => to_json_binary(&AUCTION_CONFIG.load(deps.storage)?),
        QueryMsg::Orderbook { from, limit } => to_json_binary(&query_orderbook(deps, from, limit)?),
        QueryMsg::PostedBond { solver } => to_json_binary(&query_posted_bond(deps, solver)?),
        QueryMsg::ActiveRound {} => to_json_binary(&ACTIVE_AUCTION_CONFIG.load(deps.storage)?),
        QueryMsg::AuctionPhase {} => to_json_binary(&query_active_auction_phase(deps, env)?),
    }
}

fn query_active_auction_phase(deps: QueryDeps, env: Env) -> StdResult<AuctionPhase> {
    let active_round_config = ACTIVE_AUCTION_CONFIG.load(deps.storage)?;
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
