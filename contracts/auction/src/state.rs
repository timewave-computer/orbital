use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use orbital_utils::intent::Intent;

use crate::types::{ActiveAuction, Config};

/// Config holds the configuration of the contract
pub const CONFIG: Item<Config> = Item::new("config");
/// Active auction state
pub const ACTIVE_AUCTION: Item<ActiveAuction> = Item::new("auction");
/// FIFO queue of intents to be auctioned
pub const QUEUE: cw_fifo::FIFOQueue<u64> = cw_fifo::FIFOQueue::new("front", "back", "count");
/// List of intentes we want to auction
pub const INTENTS: Map<u64, Intent> = Map::new("intents");

/// keep track of ids
pub const IDS: Item<u64> = Item::new("ids");
/// Get the next id
pub fn next_id(storage: &dyn Storage) -> StdResult<u64> {
    Ok(IDS.load(storage)? + 1)
}
