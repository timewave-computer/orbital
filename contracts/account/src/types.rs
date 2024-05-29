use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};
use orbital_utils::{domain::OrbitalDomain, intent::SavedIntent};

#[cw_serde]
pub struct QueryRecievedFundsOnDestDomain {
    pub intent: SavedIntent,
    pub winning_bid: Uint128,
    pub bidder: String,
    pub mm_addr: String,
}

#[cw_serde]
pub struct ExecuteReleaseFundsFromOrigin {
    pub origin_domain: OrbitalDomain,
    pub offer_coin: Coin,
}
