use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

use crate::domain::OrbitalDomain;

#[cw_serde]
pub struct Intent {
    pub offer_coin: Coin,
    pub ask_coin: Coin,
    pub offer_domain: OrbitalDomain,
    pub ask_domain: OrbitalDomain,
}
