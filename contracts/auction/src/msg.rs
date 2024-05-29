use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use cw_utils::Duration;
use orbital_utils::intent::Intent;

use crate::types::ActiveAuction;

#[cw_serde]
pub struct InstantiateMsg {
    pub account_addr: String,
    pub bond: Coin,
    pub increment_bps: u64, // bps
    pub duration: Duration,
    pub fulfillment_timeout: Duration,
}

#[cw_serde]
pub enum ExecuteMsg {
    Bond {},
    Slash {},
    NewIntent(Intent, String),
    AuctionTick {},
    AuctionBid {
        // Address on the ask domain (can't be verified here most of the time)
        bidder: String,
    },
    Fulfilled {
        id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ActiveAuction)]
    GetAuction {},
}
