#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{AuctionConfig, UserIntent, ADMIN, AUCTION_CONFIG, ORDERBOOK},
};

pub const CONTRACT_NAME: &str = "orbital-auction";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type QueryDeps<'a> = Deps<'a, NeutronQuery>;
pub type ExecuteDeps<'a> = DepsMut<'a, NeutronQuery>;

#[entry_point]
pub fn instantiate(
    deps: ExecuteDeps,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // set the sender as admin
    ADMIN.save(deps.storage, &info.sender)?;

    let auction_config = AuctionConfig {
        batch_size: msg.batch_size,
        auction_duration: msg.auction_duration,
        filling_window_duration: msg.filling_window_duration,
        route_config: msg.route_config,
    };
    // save the auction configuration
    AUCTION_CONFIG.save(deps.storage, &auction_config)?;

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
        ExecuteMsg::AddOrder(user_intent) => enqueue_user_intent(deps, info, user_intent),
        ExecuteMsg::FinalizeRound {} => unimplemented!(),
        ExecuteMsg::Pause {} => unimplemented!(),
        ExecuteMsg::Bid {} => unimplemented!(),
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
pub fn query(deps: QueryDeps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&ADMIN.load(deps.storage)?),
        QueryMsg::AuctionConfig {} => to_json_binary(&AUCTION_CONFIG.load(deps.storage)?),
        QueryMsg::Orderbook { from, limit } => to_json_binary(&query_orderbook(deps, from, limit)?),
    }
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
