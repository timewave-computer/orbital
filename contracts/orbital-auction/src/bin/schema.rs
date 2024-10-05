use cosmwasm_schema::write_api;
use orbital_auction::msg::{ExecuteMsg, MigrateMsg, QueryMsg};
use orbital_common::msg_types::OrbitalAuctionInstantiateMsg;

fn main() {
    write_api! {
        instantiate: OrbitalAuctionInstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        migrate: MigrateMsg,
    }
}
