use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    orbital_domain::UncheckedOrbitalDomainConfig,
    state::{ClearingAccountConfig, Transfer},
};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
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
    /// register user to orbital
    RegisterUser {},
    /// register user to a specific domain
    RegisterUserDomain { domain: String },
    /// user action to withdraw funds from their clearing account
    UserWithdrawFunds {
        // domain from which to withdraw funds
        domain: String,
        // coin to withdraw denominated in target domain
        coin: Coin,
        // target address to send funds to
        dest: String,
    },

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

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::OrbitalDomainConfig)]
    OrbitalDomain { domain: String },

    #[returns(crate::state::UserConfig)]
    UserConfig { addr: String },

    #[returns(Option<ClearingAccountConfig>)]
    ClearingAccountAddress { addr: String, domain: String },

    #[returns(neutron_sdk::interchain_queries::v047::queries::BalanceResponse)]
    Balance { query_id: u64 },

    #[returns(GetTransfersAmountResponse)]
    IcqTransfersAmount {},

    #[returns(RecipientTxsResponse)]
    IcqRecipientTxs { recipient: String },
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
