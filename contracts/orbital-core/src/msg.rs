use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

use crate::{domain::UncheckedDomainConfig, state::OrbitalDomainConfig};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    RegisterNewDomain(UncheckedDomainConfig),
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OrbitalDomainConfig)]
    OrbitalDomain { domain: String },
}
