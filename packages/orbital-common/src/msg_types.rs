use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};
use cw_utils::Duration;

#[cw_serde]
pub struct OrbitalAuctionInstantiateMsg {
    // route describes the denoms & domains involved
    pub route_config: RouteConfig,
    // auction batch size (offer denom amount)
    pub batch_size: Uint128,
    // auction phase configurations
    // duration of the bidding window in seconds
    pub auction_duration: Duration,
    // duration of the filling window in seconds
    pub filling_window_duration: Duration,
    // duration of the cleanup window in seconds
    pub cleanup_window_duration: Duration, // amount of tokens required to be posted as a slashable bond
    // in order to participate in the auction
    pub solver_bond: Coin,
}

#[cw_serde]
pub struct RouteConfig {
    pub src_domain: String,
    pub dest_domain: String,
    pub offer_denom: String,
    pub ask_denom: String,
}
