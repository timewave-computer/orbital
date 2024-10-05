use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};
use orbital_common::msg_types::OrbitalAuctionInstantiateMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// code id of the orbital auction contract
pub(crate) const ORBITAL_AUCTION_CODE_ID: Item<Uint64> = Item::new("code_id");

/// orbital auction nonce
pub(crate) const ORBITAL_AUCTION_NONCE: Item<Uint64> = Item::new("auction_nonce");

/// registered orbital auctions
pub(crate) const ORBITAL_AUCTIONS: Map<u64, OrbitalAuctionConfig> = Map::new("auctions");

/// keeping track of registered user IDs which get incremented
/// with each new registration. it's needed to generate unique
/// user clearing account identifiers.
pub(crate) const USER_NONCE: Item<Uint64> = Item::new("user_nonce");

/// map of users with their respective configurations
pub(crate) const USER_CONFIGS: Map<String, UserConfig> = Map::new("user_configs");

/// map of registered remote domains and their configuration
pub(crate) const ORBITAL_DOMAINS: Map<String, OrbitalDomainConfig> = Map::new("domains");

/// map of clearing accounts registered with orbital.
/// key is a composite of (user_id, domain) generated with
/// `utils::get_ica_identifier`. value is an optional address where:
/// - None: clearing account is being registered and awaiting callback
/// - Some: clearing account has been registered and is ready for use
pub(crate) const CLEARING_ACCOUNTS: Map<String, Option<ClearingAccountConfig>> =
    Map::new("clearing_accounts");

/// contains all transfers mapped by a recipient address observed by the contract.
pub(crate) const RECIPIENT_TXS: Map<String, Vec<Transfer>> = Map::new("recipient_txs");
/// contains number of transfers to addresses observed by the contract.
pub(crate) const TRANSFERS: Item<u64> = Item::new("transfers");

#[cw_serde]
pub struct OrbitalAuctionConfig {
    pub src_domain: String,
    pub src_clearing_acc_id: String,
    pub src_clearing_acc_addr: Option<String>,
    pub dest_domain: String,
    pub dest_clearing_acc_id: String,
    pub dest_clearing_acc_addr: Option<String>,
    pub auction_addr: Option<String>,
    pub orbital_auction_instantiate_msg: OrbitalAuctionInstantiateMsg,
}

impl OrbitalAuctionConfig {
    pub fn prepared_clearing_accounts(&self) -> bool {
        self.src_clearing_acc_addr.is_some() && self.dest_clearing_acc_addr.is_some()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Transfer {
    pub recipient: String,
    pub sender: String,
    pub denom: String,
    pub amount: String,
}

#[cw_serde]
pub struct ClearingAccountConfig {
    pub addr: String,
    pub controller_connection_id: String,
}

#[cw_serde]
pub struct UserConfig {
    pub id: Uint64,
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
