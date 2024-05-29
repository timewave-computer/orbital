use cosmwasm_std::{Addr, Coin, Empty, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use orbital_utils::intent::{Intent, SavedIntent};

use crate::types::{ActiveAuction, Config};

/// Config holds the configuration of the contract
pub const CONFIG: Item<Config> = Item::new("config");
pub const BONDS: Map<Addr, Coin> = Map::new("bonds");
/// Active auction state
pub const ACTIVE_AUCTION: Item<ActiveAuction> = Item::new("auction");
/// FIFO queue of intents to be auctioned
pub const QUEUE: cw_fifo::FIFOQueue<u64> = cw_fifo::FIFOQueue::new("front", "back", "count");
/// List of intentes we want to auction
pub const INTENTS: Map<u64, SavedIntent> = Map::new("intents");

pub const TO_VERIFY: Item<ActiveAuction> = Item::new("to_verify");
/// keep track of ids
pub const IDS: Item<u64> = Item::new("ids");
/// Get the next id
pub fn next_id(storage: &dyn Storage) -> StdResult<u64> {
    Ok(IDS.load(storage)? + 1)
}
