use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const DENOM_FALLBACK: &str = "ufallback";
pub const DENOM_ATOM: &str = "uatom";
pub const DENOM_NTRN: &str = "untrn";
pub const DENOM_OSMO: &str = "uosmo";
pub const FAUCET: &str = "faucet_addr";
pub const ADMIN: &str = "admin_addr";
pub const ALL_DENOMS: &[&str] = &[DENOM_ATOM, DENOM_NTRN, DENOM_OSMO, DENOM_FALLBACK];
pub const CHAIN_PREFIX: &str = "cosmos";
pub const OWNER: &str = "owner";
pub const NOTE: &str = "note";
pub const USER_1: &str = "user_1";
pub const SOLVER: &str = "solver";
pub const SOLVER_2: &str = "solver_2";

pub const GAIA_DOMAIN: &str = "gaia";
pub const OSMOSIS_DOMAIN: &str = "osmosis";

/// Namespace for neutron storage
pub const NAMESPACE_NEUTRON: &[u8] = b"neutron_storage";

/// Map for (sender, conn_id) => account_id
pub const ACCOUNTS: Map<(&Addr, String, String), Addr> = Map::new("accounts");

pub const LOCAL_CHANNELS: Map<String, String> = Map::new("local_channels");
pub const LOCAL_CHANNELS_VALUES: Map<String, String> = Map::new("local_channels_values");

pub const REMOTE_CHANNELS: Map<String, String> = Map::new("remote_channels");
pub const REMOTE_CHANNELS_VALUES: Map<String, String> = Map::new("remote_channels_values");
