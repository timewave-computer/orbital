use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw_utils::{Duration, Expiration};
use orbital_utils::domain::OrbitalDomain;

#[cw_serde]
pub struct Config {
    pub account_addr: Addr,
    pub bond_amount: Coin,
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
    pub highest_bid: Uint128,
    pub bidder: Option<String>,
    pub end_time: Expiration,
}
