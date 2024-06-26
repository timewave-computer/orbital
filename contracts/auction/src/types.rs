use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw_utils::{Duration, Expiration};
use orbital_utils::{
    domain::OrbitalDomain,
    intent::{Intent, SavedIntent},
};

#[cw_serde]
pub struct Config {
    pub account_addr: Addr,
    pub bond: Coin,
    pub increment_decimal: Decimal, // bps
    pub duration: Duration,
    pub fulfillment_timeout: Duration,
}

#[cw_serde]
pub struct NewAuction {
    offer_coin: Coin,
    ask_denom: String,
    offer_domain: OrbitalDomain,
    ask_deomain: OrbitalDomain,
}

#[cw_serde]
pub struct ActiveAuction {
    pub intent_id: u64,
    pub highest_bid: Coin,
    pub bidder: Option<String>,
    pub mm_addr: Option<String>,
    pub end_time: Expiration,
    pub verified: bool,
}

#[cw_serde]
pub struct WaitingFulfillment {
    pub id: u64,
    pub original_intent: Intent,
    pub winning_bid: Uint128,
    pub bidder: String,
    pub fulfilled: bool,
}

#[cw_serde]
pub enum TestAccountExecuteMsg {
    VerifyAuction {
        original_intent: SavedIntent,
        winning_bid: Uint128,
        bidder: String,
        mm_addr: String,
    },
}
