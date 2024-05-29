use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use orbital_utils::intent::{Intent, SavedIntent};

#[cw_serde]
pub struct QueryRecievedFundsOnDestDomain {
    pub intent: SavedIntent,
    pub winning_bid: Uint128,
    pub bidder: String,
}
