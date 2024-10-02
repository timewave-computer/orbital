use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};
use cw_utils::Duration;

use crate::state::{AuctionPhase, RouteConfig, UserIntent};

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
    pub cleanup_window_duration: Duration,
    // amount of tokens required to be posted as a slashable bond
    // in order to participate in the auction
    pub solver_bond: Coin,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// adds an order to the auction to be executed on the next round.
    /// only callable by orbital-core which is responsible for escrowing orders.
    AddOrder(UserIntent),
    /// finalizes the current auction round and prepares for the next
    Tick { mock_fill_status: bool },
    /// pause the auction, stopping any new orders from being accepted
    Pause {},
    /// resume the auction, allowing new orders to be accepted
    Resume {},

    // bidder actions
    /// post a bond to participate in the auction
    PostBond {},

    /// withdraw the posted bond
    WithdrawBond {},

    /// bid on the current auction
    Bid { amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cosmwasm_std::Addr)]
    Admin {},

    #[returns(crate::state::AuctionConfig)]
    AuctionConfig {},

    #[returns(Vec<UserIntent>)]
    Orderbook {
        from: Option<u32>,
        limit: Option<u32>,
    },

    #[returns(Coin)]
    PostedBond { solver: String },

    #[returns(crate::state::AuctionRound)]
    ActiveRound {},

    #[returns(AuctionPhase)]
    AuctionPhase {},
}

#[cw_serde]
pub enum MigrateMsg {}
