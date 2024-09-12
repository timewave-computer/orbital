use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::Map;

/// map of users with their respective configurations
pub const USER_CONFIGS: Map<Addr, UserConfig> = Map::new("user_configs");

#[cw_serde]
#[derive(Default)]
pub struct UserConfig {
    // TODO: this may make more sense to have as a top level map
    // with composite keys of (user, domain) -> clearing_addr
    pub clearing_accounts: HashMap<String, String>,
}


/// map of registered remote domains and their configuration
pub const ORBITAL_DOMAINS: Map<String, OrbitalDomainConfig> = Map::new("domains");

/// remote domain configuration config which supports different types of account implementations.
/// currently supported types:
/// - Polytone: cw-based account implementation that operates via note contract on the origin chain
/// - ICA: interchain account implementation based on ICS-27
#[cw_serde]
pub enum OrbitalDomainConfig {
    Polytone {
        note: Addr,
        timeout: Uint64,
    },
    ICA {
        connection_id: String,
        channel_id: String,
        timeout: Uint64,
    },
}
