use std::collections::HashMap;

use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use orbital_utils::domain::OrbitalDomain;

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


pub const POLYTONE_NOTES: Map<u8, Addr> = Map::new("polytone_notes");

#[cw_serde]
pub struct UserConfig {
    pub registered_domains: HashMap<u8, Addr>,
}

pub const USER_CONFIGS: Map<Addr, UserConfig> = Map::new("user_configs");

pub const USER_DOMAINS: Map<Addr, Vec<OrbitalDomain>> = Map::new("user_domains");