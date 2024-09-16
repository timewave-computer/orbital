use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::Map;

/// map of registered remote domains and their configuration
pub const ORBITAL_DOMAINS: Map<String, OrbitalDomainConfig> = Map::new("domains");

/// remote domain configuration config which supports different types of account implementations.
/// currently supported types:
/// - Polytone: cw-based account implementation that operates via note contract on the origin chain
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
