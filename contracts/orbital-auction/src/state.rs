use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Coin, StdResult, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Deque, Item, Map};
use cw_utils::{Duration, Expiration};

// authorized orbital-core address
pub const ADMIN: Item<Addr> = Item::new("admin");

// identifier for current auction. advances on successful round.
pub const AUCTION_ID: Item<Uint64> = Item::new("auction_id");

// global auction configuration that applies for every round
pub const AUCTION_CONFIG: Item<AuctionConfig> = Item::new("auction_config");

// current batch configuration
pub const ACTIVE_AUCTION_CONFIG: Item<ActiveRoundConfig> = Item::new("current_round_config");

// map of solvers registered for participating in the auction
pub const POSTED_BONDS: Map<String, Coin> = Map::new("posted_bonds");

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
    // duration of the cleanup window in seconds
    pub cleanup_window_duration: Duration,
    // config that describes the route for the auction
    // (src & dest domains, offer & ask denoms)
    pub route_config: RouteConfig,
    // configured bond amount required to participate in the auction
    pub solver_bond: Coin,
}

impl AuctionConfig {
    /// returns the total duration of a round (in seconds),
    /// which is the sum of the auction, filling, and cleaning window durations
    pub fn get_total_round_duration(&self) -> StdResult<Duration> {
        (self.auction_duration + self.filling_window_duration)? + self.cleanup_window_duration
    }
}

#[cw_serde]
pub struct ActiveRoundConfig {
    pub id: Uint64,
    pub start_time: Timestamp,
    pub end_time: Expiration,
    pub batch: BatchStatus,
}

#[cw_serde]
pub enum BatchStatus {
    Empty {},
    Active {
        user_intents: Vec<UserIntent>,
        current_bid: Bid,
    },
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
