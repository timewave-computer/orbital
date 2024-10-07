use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Uint128};

// base definition of an order. will likely change.
#[cw_serde]
pub struct UserIntent {
    pub user: String,
    pub amount: Uint128,
    pub offer_domain: String,
    pub ask_domain: String,
}

impl UserIntent {
    /// splits the order into two orders, one with the given amount and the remainder.
    /// returns an error if the amount exceeds the order amount. if it doesn't, returns
    /// a tuple in the form of (new_order, remainder).
    pub fn split_order(&self, amount: Uint128) -> StdResult<(UserIntent, UserIntent)> {
        let new_order = UserIntent {
            user: self.user.to_string(),
            amount,
            offer_domain: self.offer_domain.to_string(),
            ask_domain: self.ask_domain.to_string(),
        };

        let remainder = UserIntent {
            user: self.user.to_string(),
            amount: self.amount.checked_sub(amount)?,
            offer_domain: self.offer_domain.to_string(),
            ask_domain: self.ask_domain.to_string(),
        };

        Ok((new_order, remainder))
    }
}
