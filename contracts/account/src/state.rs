use std::collections::HashMap;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use orbital_utils::domain::OrbitalDomain;

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const AUCTION_ADDR: Item<Addr> = Item::new("auctoin_addr");

/// double accounting system
pub const LEDGER: Map<u8, HashMap<String, u128>> = Map::new("ledger");

/// active user domains (domain -> addr)
pub const USER_DOMAINS: Map<u8, String> = Map::new("user_domain_configs");

/// registered domain notes
pub const NOTE_TO_DOMAIN: Map<Addr, OrbitalDomain> = Map::new("note_to_domain");
pub const DOMAIN_TO_NOTE: Map<u8, Addr> = Map::new("domain_to_note");
