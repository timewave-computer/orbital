use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub polytone_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateNewIntent {},
    
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetClaimable {},
}
