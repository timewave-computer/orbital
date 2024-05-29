use cosmwasm_schema::{cw_serde, QueryResponses};
use orbital_utils::domain::OrbitalDomain;
use polytone::callbacks::CallbackMessage;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterDomain {
        domain: OrbitalDomain,
        note_addr: String,
    },
    // polytone callback listener
    Callback(CallbackMessage),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
