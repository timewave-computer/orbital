use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coins, Addr, DepsMut, StdResult, Uint64};
use cw_storage_plus::Map;
use neutron_sdk::bindings::msg::NeutronMsg;

/// map of users with their respective configurations
pub const USER_CONFIGS: Map<Addr, UserConfig> = Map::new("user_configs");

/// map of registered remote domains and their configuration
pub const ORBITAL_DOMAINS: Map<String, OrbitalDomainConfig> = Map::new("domains");

/// map of clearing accounts registered with orbital.
/// key is a composite of (domain_identifier, owner_neutron_addr).
/// value is an optional address where:
/// - None: clearing account is being registered and awaiting callback
/// - Some: clearing account has been registered and is ready for use
pub const CLEARING_ACCOUNTS: Map<(String, String), Option<Addr>> = Map::new("clearing_accounts");

#[cw_serde]
#[derive(Default)]
pub struct UserConfig {}

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

impl OrbitalDomainConfig {
    pub fn get_registration_message(
        &self,
        deps: DepsMut,
        domain: String,
        user_addr: Addr,
    ) -> StdResult<NeutronMsg> {
        let msg = match self {
            OrbitalDomainConfig::InterchainAccount { connection_id, .. } => {
                NeutronMsg::register_interchain_account(
                    connection_id.to_string(),
                    user_addr.to_string(),
                    Some(coins(100_000, "untrn")),
                )
            }
            _ => unimplemented!(),
        };

        // store `None` as the clearing account until the callback is received
        CLEARING_ACCOUNTS.save(deps.storage, (domain, user_addr.to_string()), &None)?;

        Ok(msg)
    }
}
