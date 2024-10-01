use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Coin, StdResult, Uint128, Uint64};
use cw_storage_plus::{Deque, Item, Map};
use cw_utils::{Duration, Expiration};

// authorized orbital-core address
pub const ADMIN: Item<Addr> = Item::new("admin");

// identifier for current auction. advances on successful round.
pub const AUCTION_ID: Item<Uint64> = Item::new("auction_id");

// global auction configuration that applies for every round
pub const AUCTION_CONFIG: Item<AuctionConfig> = Item::new("auction_config");

// current batch configuration
pub const ACTIVE_AUCTION_CONFIG: Item<AuctionRound> = Item::new("current_round_config");

// archive of past auction rounds
pub const AUCTION_ARCHIVE: Deque<AuctionRound> = Deque::new("auction_archive");

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
    // config that describes the time durations for each phase of the auction
    pub auction_phases: AuctionPhaseConfig,
    // config that describes the route for the auction
    // (src & dest domains, offer & ask denoms)
    // TODO: is this relevant?
    pub route_config: RouteConfig,
    // configured bond amount required to participate in the auction
    pub solver_bond: Coin,
}

/// orbital auction operates in discrete-time based rounds.
/// each round consists of the following phases, in order:
/// - auction: bidding window where solvers can submit their bids
/// - filling: window where the auction is finalized and orders are matched
/// - cleanup: window where the auction is reset and the next round is prepared
// TODO: validate that all durations are passed in seconds.
// or just drop Duration altogether and deal in seconds?
#[cw_serde]
pub struct AuctionPhaseConfig {
    // duration of the bidding window in seconds
    pub auction_duration: Duration,
    // duration of the filling window in seconds
    pub filling_window_duration: Duration,
    // duration of the cleanup window in seconds
    pub cleanup_window_duration: Duration,
}

impl AuctionPhaseConfig {
    /// returns the total duration of a round (in seconds),
    /// which is the sum of the auction, filling, and cleaning window durations
    pub fn get_total_round_duration(&self) -> StdResult<Duration> {
        (self.auction_duration + self.filling_window_duration)? + self.cleanup_window_duration
    }
}

#[cw_serde]
pub struct AuctionRound {
    pub id: Uint64,
    pub phases: RoundPhaseExpirations,
    pub batch: BatchStatus,
}

#[cw_serde]
pub struct RoundPhaseExpirations {
    pub start_expiration: Expiration,
    pub auction_expiration: Expiration,
    pub filling_expiration: Expiration,
    pub cleanup_expiration: Expiration,
}

impl RoundPhaseExpirations {
    /// returns the absolute expiration configuration given the current block time and the
    /// relative auction phase configuration.
    pub fn from_auction_config(value: &AuctionPhaseConfig, block: &BlockInfo) -> StdResult<Self> {
        // phases advance in order of auction -> filling -> cleanup.
        // first we derive the durations with respect to t_0 which is block.time
        // (auction duration is already the needed delta).
        let filling_duration_from_t0 = (value.auction_duration + value.filling_window_duration)?;
        let cleanup_duration_from_t0 = (filling_duration_from_t0 + value.cleanup_window_duration)?;

        // then we calculate the absolute expiration times for each phase
        let phases = RoundPhaseExpirations {
            start_expiration: Expiration::AtTime(block.time),
            auction_expiration: value.auction_duration.after(block),
            filling_expiration: filling_duration_from_t0.after(block),
            cleanup_expiration: cleanup_duration_from_t0.after(block),
        };

        Ok(phases)
    }

    /// given a block, compares the block timestamp to the phase expirations and returns
    /// the current phase of the auction.
    pub fn get_current_phase(&self, block: &BlockInfo) -> AuctionPhase {
        if self.cleanup_expiration.is_expired(block) {
            AuctionPhase::OutOfSync
        } else if self.filling_expiration.is_expired(block) {
            AuctionPhase::Cleanup
        } else if self.auction_expiration.is_expired(block) {
            AuctionPhase::Filling
        } else {
            AuctionPhase::Bidding
        }
    }
}

#[cw_serde]
pub enum AuctionPhase {
    Bidding,
    Filling,
    Cleanup,
    OutOfSync,
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
