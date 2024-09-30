use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_utils::Duration;

use crate::state::RouteConfig;

#[cw_serde]
pub struct InstantiateMsg {
    // config that describes the route:
    // - src & destination domains
    // - offer & ask denoms
    pub route_config: RouteConfig,
    // auction batch size (offer denom amount)
    pub batch_size: Uint128,
    // auction time configurations expressed in seconds
    pub auction_duration: Duration,
    pub filling_window_duration: Duration,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// adds an order to the auction to be executed on the next round.
    /// only callable by orbital-core which is responsible for escrowing orders.
    AddOrder {},
    /// finalizes the current auction round and prepares for the next
    FinalizeRound {},
    /// pause the auction, stopping any new orders from being accepted
    Pause {},

    // bidder actions
    /// bid on the current auction
    Bid {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cosmwasm_std::Addr)]
    Admin {},

    #[returns(crate::state::AuctionConfig)]
    AuctionConfig {},
}

#[cw_serde]
pub enum MigrateMsg {}
