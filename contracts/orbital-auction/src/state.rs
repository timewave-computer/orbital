use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Deque, Item};
use cw_utils::Duration;

// authorized orbital-core address
pub const ADMIN: Item<Addr> = Item::new("admin");

// identifier for current auction. advances on successful round.
pub const AUCTION_ID: Item<Uint64> = Item::new("auction_id");

// global auction configuration that applies for every round
pub const AUCTION_CONFIG: Item<AuctionConfig> = Item::new("auction_config");

// current batch configuration
pub const CURRENT_BATCH_CONFIG: Item<AuctionBatch> = Item::new("current_round_config");

// orderbook is the queue of orders to be included in the next auction.
// orders are processed in a FIFO manner. if an order cannot be entirely
// included in the auction, order is split and the remainder is re-enqueued,
// maintaining the priority.
pub const ORDERBOOK: Deque<UserIntent> = Deque::new("orderbook");

// base definition of an order. will likely change.
#[cw_serde]
pub struct UserIntent {
    pub user: String,
    pub amount: Uint128,
    pub offer_domain: String,
    pub ask_domain: String,
}

#[cw_serde]
pub struct AuctionConfig {
    // how many of the offer denom we can fit in a batch
    pub batch_size: Uint128,
    // duration of the bidding window in seconds
    pub auction_duration: Duration,
    // duration of the filling window in seconds
    pub filling_window_duration: Duration,
    // config that describes the route for the auction
    // (src & dest domains, offer & ask denoms)
    pub route_config: RouteConfig,
}

#[cw_serde]
pub struct AuctionBatch {
    pub user_intents: Vec<UserIntent>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub current_bid: Bid,
}

#[cw_serde]
pub struct Bid {
    pub solver: Addr,
    pub amount: Uint128,
    pub bid_block: BlockInfo,
}

#[cw_serde]
pub struct RouteConfig {
    pub src_domain: String,
    pub dest_domain: String,
    pub offer_denom: String,
    pub ask_denom: String,
}
