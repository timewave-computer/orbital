use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};
use orbital_common::intent::UserIntent;

use crate::state::AuctionPhase;

#[cw_serde]
pub enum ExecuteMsg {
    /// adds an order to the auction to be executed on the next round.
    /// only callable by orbital-core which is responsible for escrowing orders.
    AddOrder(UserIntent),
    /// finalizes the current auction round and prepares for the next
    Tick { mock_fill_status: bool },
    /// pause the auction, stopping any new orders from being accepted
    Pause {},
    /// resume the auction, allowing new orders to be accepted
    Resume {},

    // bidder actions
    /// post a bond to participate in the auction
    PostBond {},

    /// withdraw the posted bond
    WithdrawBond {},

    /// bid on the current auction
    Bid { amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cosmwasm_std::Addr)]
    Admin {},

    #[returns(crate::state::AuctionConfig)]
    AuctionConfig {},

    #[returns(Vec<UserIntent>)]
    Orderbook {
        from: Option<u32>,
        limit: Option<u32>,
    },

    #[returns(Coin)]
    PostedBond { solver: String },

    #[returns(crate::state::AuctionRound)]
    ActiveRound {},

    #[returns(AuctionPhase)]
    AuctionPhase {},
}

#[cw_serde]
pub enum MigrateMsg {}
