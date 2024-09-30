#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{AuctionConfig, UserIntent, ADMIN, AUCTION_CONFIG, ORDERBOOK, POSTED_BONDS},
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
        solver_bond: msg.solver_bond,
    };

    // save the auction configuration
    AUCTION_CONFIG.save(deps.storage, &auction_config)?;

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
        ExecuteMsg::AddOrder(user_intent) => enqueue_user_intent(deps, info, user_intent),
        ExecuteMsg::FinalizeRound {} => unimplemented!(),
        ExecuteMsg::Pause {} => unimplemented!(),
        ExecuteMsg::Bid {} => unimplemented!(),
        ExecuteMsg::PostBond {} => try_post_bond(deps, info),
    }
}

fn try_post_bond(deps: ExecuteDeps, info: MessageInfo) -> NeutronResult<Response<NeutronMsg>> {
    let auction_config = AUCTION_CONFIG.load(deps.storage)?;

    println!("funds sent: {:?}", info.funds);
    // get the amount of tokens sent by the solver
    let posted_bond_amount = must_pay(&info, &auction_config.solver_bond.denom)
        .map_err(ContractError::FeePaymentError)?;
    println!("posted bond amount: {:?}", posted_bond_amount);
    // depending on if this is the first time the solver is posting a bond,
    // or if they have already posted a bond before, we return the total
    let new_bond = match POSTED_BONDS.may_load(deps.storage, info.sender.to_string())? {
        Some(existing_bond) => coin(
            existing_bond.amount.checked_add(posted_bond_amount)?.u128(),
            existing_bond.denom,
        ),
        None => coin(posted_bond_amount.u128(), &auction_config.solver_bond.denom),
    };

    POSTED_BONDS.save(deps.storage, info.sender.to_string(), &new_bond)?;

    Ok(Response::default())
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
        QueryMsg::PostedBond { solver } => to_json_binary(&query_posted_bond(deps, solver)?),
    }
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
