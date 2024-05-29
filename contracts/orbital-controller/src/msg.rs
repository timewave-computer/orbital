use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use orbital_utils::domain::OrbitalDomain;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    // global configs
    RegisterDomain {
        domain: OrbitalDomain,
        note_addr: String,
    },

    // user configs
    RegisterUser {
        domains: Vec<OrbitalDomain>,
    },
    RegisterUserDomain {
        domain: OrbitalDomain,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetClaimable {},
}
