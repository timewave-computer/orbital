use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::{
    contract::{execute, instantiate, query},
    msg::InstantiateMsg,
};

const OWNER: &str = "owner";
const NOTE: &str = "note";

pub fn get_orbital_core_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

pub struct Suite {
    pub app: App,
    pub owner: Addr,
    pub orbital: Addr,
    pub note: Addr,
}

fn make_addr(app: &App, addr: &str) -> Addr {
    app.api().addr_make(addr)
}

impl Default for Suite {
    fn default() -> Self {
        let mut app = App::default();

        let code_id = app.store_code(get_orbital_core_contract());
        let owner_addr = make_addr(&app, OWNER);
        let note_addr = make_addr(&app, NOTE);

        let addr = app
            .instantiate_contract(
                code_id,
                owner_addr.clone(),
                &InstantiateMsg {
                    owner: owner_addr.to_string(),
                },
                &[],
                "orbital-core",
                None,
            )
            .unwrap();

        Self {
            app,
            owner: owner_addr,
            orbital: addr,
            note: note_addr,
        }
    }
}
