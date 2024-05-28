use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub denom: String,
    pub receiver: String, // address (ibc or not)
    pub claimer: Addr,
    pub start: Timestamp,
    pub end: Timestamp,
    pub ibc_channel_id: Option<String>,
}
