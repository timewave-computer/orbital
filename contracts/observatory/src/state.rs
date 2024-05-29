use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const USER_ADDR: Item<Addr> = Item::new("user_addr");
pub const NOTES: Map<u8, Addr> = Map::new("notes");
