use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_utils::Expiration;
use orbital_utils::{domain::OrbitalDomain, intent::Intent};
use polytone::callbacks::CallbackMessage;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterDomain {
        domain: OrbitalDomain,
        note_addr: String,
    },
    // polytone callback listener
    Callback(CallbackMessage),
    Sync {
        domain: OrbitalDomain,
    },
    UpdateAuctionAddr {
        auction_addr: String,
    },
    NewIntent(Intent),
    AuctionFinished {
        original_intent: Intent,
        fulfillment_end: Expiration,
        id: u64,
        winning_bid: Uint128,
        bidder: Option<String>,
    },
    VerifyAuction {
        id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
