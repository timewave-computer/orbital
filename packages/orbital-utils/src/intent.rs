use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

use crate::domain::OrbitalDomain;

#[cw_serde]
pub struct Intent {
    pub offer_coin: Coin,
    pub ask_coin: Coin,
    pub offer_domain: OrbitalDomain,
    pub ask_domain: OrbitalDomain,
    pub is_verified: bool,
}
impl Intent {
    pub fn into_saved_intent(self, deposit_addr: String) -> SavedIntent {
        SavedIntent {
            deposit_addr,
            offer_coin: self.offer_coin,
            ask_coin: self.ask_coin,
            offer_domain: self.offer_domain,
            ask_domain: self.ask_domain,
            is_verified: self.is_verified,
        }
    }
}

#[cw_serde]
pub struct SavedIntent {
    pub deposit_addr: String,
    pub offer_coin: Coin,
    pub ask_coin: Coin,
    pub offer_domain: OrbitalDomain,
    pub ask_domain: OrbitalDomain,
    pub is_verified: bool,
}