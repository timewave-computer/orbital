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
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    solver,
    state::{
        ActiveRoundConfig, AuctionConfig, AuctionPhaseConfig, BatchStatus, RoundPhaseExpirations,
        UserIntent, ACTIVE_AUCTION_CONFIG, ADMIN, AUCTION_CONFIG, ORDERBOOK, POSTED_BONDS,
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

    // set the sender as admin
    ADMIN.save(deps.storage, &info.sender)?;

    // save the auction configuration
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
    AUCTION_CONFIG.save(deps.storage, &auction_config)?;

    // start the auction cycle with an empty batch
    let active_round_config = ActiveRoundConfig {
        id: Uint64::zero(),
        batch: BatchStatus::Empty {},
        start_time: env.block.time,
        phases: RoundPhaseExpirations::from_auction_config(
            auction_config.auction_phases,
            &env.block,
        )?,
    };
    ACTIVE_AUCTION_CONFIG.save(deps.storage, &active_round_config)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: ExecuteDeps,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        // user action
        ExecuteMsg::AddOrder(user_intent) => enqueue_user_intent(deps, info, user_intent),

        // permisionless action
        ExecuteMsg::FinalizeRound {} => unimplemented!(),

        // solver actions
        ExecuteMsg::Bid {} => unimplemented!(),
        ExecuteMsg::PostBond {} => solver::try_post_bond(deps, info),
        ExecuteMsg::WithdrawBond {} => solver::try_withdraw_posted_bond(deps, info),
        ExecuteMsg::Prove {} => unimplemented!(),

        // admin-gated actions
        ExecuteMsg::Pause {} => unimplemented!(),
        ExecuteMsg::Resume {} => unimplemented!(),
    }
}

/// admin-gated action to include a (validated) user intent into the orderbook.
fn enqueue_user_intent(
    deps: ExecuteDeps,
    info: MessageInfo,
    user_intent: UserIntent,
) -> NeutronResult<Response<NeutronMsg>> {
    // only the admin can enqueue new orders on behalf of the users
    ensure!(
        info.sender == ADMIN.load(deps.storage)?,
        ContractError::Unauthorized {}
    );

    // add the user intent to the end of the orderbook queue
    ORDERBOOK.push_back(deps.storage, &user_intent)?;

    Ok(Response::default())
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

fn query_active_auction_phase(deps: QueryDeps, env: Env) -> StdResult<String> {
    let active_round_config = ACTIVE_AUCTION_CONFIG.load(deps.storage)?;
    let phase = active_round_config.phases.get_current_phase(&env.block);

    Ok(phase.to_string())
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
