use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};
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
    VerifyAuction {
        original_intent: Intent,
        winning_bid: Uint128,
        bidder: String,
    },
    WithdrawFunds {
        domain: OrbitalDomain,
        coin: Coin,
        dest: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    QueryDomainAddr {
        domain: String,
    },
    #[returns(String)]
    QueryProxyAccount {
        domain: String,
    },
    #[returns(Vec<(String, u128)>)]
    QueryLedger {
        domain: String,
    },
    #[returns(String)]
    QueryAllDomains{},
    #[returns(Vec<(String, String, u128)>)]
    QueryAllLedgers{},
}
