use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

use crate::account_types::UncheckedOrbitalDomainConfig;

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
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::OrbitalDomainConfig)]
    OrbitalDomain { domain: String },

    #[returns(crate::state::UserConfig)]
    UserConfig { user: String },
}
