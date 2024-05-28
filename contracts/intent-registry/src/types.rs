use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub polytone_addr: Addr,
}

#[cw_serde]
pub struct Intent {
    pub polytone_addr: Addr,
}
