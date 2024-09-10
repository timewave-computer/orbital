use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::Map;

/// map of registered remote domains and their configuration
pub const ORBITAL_DOMAINS: Map<String, OrbitalDomainConfig> = Map::new("domains");

/// remote domain configuration config which supports different types of bridge connections.
/// currently supported types:
/// - Polytone: cw-based bridge that operates via note contract on the origin chain
#[cw_serde]
pub enum OrbitalDomainConfig {
    Polytone {
        domain: String,
        note: Addr,
        timeout: Uint64,
    },
}
