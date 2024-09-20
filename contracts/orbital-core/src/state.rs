use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::Map;

/// map of users with their respective configurations
pub const USER_CONFIGS: Map<String, UserConfig> = Map::new("user_configs");

/// map of registered remote domains and their configuration
pub const ORBITAL_DOMAINS: Map<String, OrbitalDomainConfig> = Map::new("domains");

/// map of clearing accounts registered with orbital.
/// key is a composite of (domain_identifier, owner_neutron_addr).
/// value is an optional address where:
/// - None: clearing account is being registered and awaiting callback
/// - Some: clearing account has been registered and is ready for use
pub const CLEARING_ACCOUNTS: Map<String, Option<String>> = Map::new("clearing_accounts");

#[cw_serde]
#[derive(Default)]
pub struct UserConfig {
    pub registered_domains: Vec<String>,
}

/// remote domain configuration config which supports different types of account implementations.
/// currently supported types:
/// - Polytone: cw-based account implementation that operates via note contract on the origin chain
/// - InterchainAccount: interchain account implementation based on ICS-27
#[cw_serde]
pub enum OrbitalDomainConfig {
    Polytone {
        note: Addr,
        timeout: Uint64,
    },
    InterchainAccount {
        connection_id: String,
        channel_id: String,
        timeout: Uint64,
    },
}
