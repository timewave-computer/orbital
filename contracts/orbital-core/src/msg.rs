use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128, Uint64};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use orbital_common::msg_types::{OrbitalAuctionInstantiateMsg, RouteConfig};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{orbital_domain::UncheckedOrbitalDomainConfig, state::Transfer};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub auction_code_id: Uint64,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// admin-gated action to enable new orbital domains
    RegisterNewDomain {
        // string identifier for the domain
        domain: String,
        // type of account to be used
        account_type: UncheckedOrbitalDomainConfig,
    },
    /// enable new auction
    RegisterNewAuction(OrbitalAuctionInstantiateMsg),
    /// register user to orbital
    RegisterUser {},
    /// register user to a specific domain
    RegisterUserDomain {
        domain: String,
    },
    /// user action to withdraw funds from their clearing account
    UserWithdrawFunds {
        // domain from which to withdraw funds
        domain: String,
        // coin to withdraw denominated in target domain
        coin: Coin,
        // target address to send funds to
        dest: String,
    },
    SubmitIntent(SubmitIntentMsg),
    // ICQ related messages
    RegisterBalancesQuery {
        connection_id: String,
        update_period: u64,
        addr: String,
        denoms: Vec<String>,
    },
    RegisterTransfersQuery {
        connection_id: String,
        update_period: u64,
        recipient: String,
        min_height: Option<u64>,
    },
}

#[cw_serde]
pub struct SubmitIntentMsg {
    pub route_config: RouteConfig,
    pub amount: Uint128,
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::OrbitalDomainConfig)]
    OrbitalDomain { domain: String },

    #[returns(crate::state::UserConfig)]
    UserConfig { addr: String },

    #[returns(Option<crate::state::ClearingAccountConfig>)]
    ClearingAccountAddress { addr: String, domain: String },

    #[returns(Option<crate::state::ClearingAccountConfig>)]
    AuctionClearingAccountAddress { id: Uint64, domain: String },

    #[returns(neutron_sdk::interchain_queries::v047::queries::BalanceResponse)]
    Balance { query_id: u64 },

    #[returns(GetTransfersAmountResponse)]
    IcqTransfersAmount {},

    #[returns(RecipientTxsResponse)]
    IcqRecipientTxs { recipient: String },

    #[returns(crate::state::OrbitalAuctionConfig)]
    Auction { id: Uint64 },

    #[returns(Vec<String>)]
    ReplyDebugLog {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetTransfersAmountResponse {
    pub transfers_number: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RecipientTxsResponse {
    pub transfers: Vec<Transfer>,
}

#[cw_serde]
pub enum MigrateMsg {}
