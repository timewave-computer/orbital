#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{AuctionConfig, ADMIN, AUCTION_CONFIG},
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
    unimplemented!()
}

#[entry_point]
pub fn query(deps: QueryDeps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[entry_point]
pub fn reply(_deps: ExecuteDeps, _env: Env, _msg: Reply) -> StdResult<Response<NeutronMsg>> {
    unimplemented!()
}

#[entry_point]
pub fn migrate(_deps: ExecuteDeps, _env: Env, _msg: MigrateMsg) -> StdResult<Response<NeutronMsg>> {
    unimplemented!()
}
