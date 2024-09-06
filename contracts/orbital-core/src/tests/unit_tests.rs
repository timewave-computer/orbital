use cosmwasm_std::Addr;
use cw_ownable::Ownership;

use crate::{contract::{execute, instantiate, query}, msg::{InstantiateMsg, QueryMsg}};
use cw_multi_test::{App, ContractWrapper, Executor};

const OWNER: &str = "owner";

fn get_owner_addr(app: &App) -> Addr {
    app.api().addr_make(OWNER)
}

#[test]
fn test_init() {   
    let mut app = App::default();
    
    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));
    let owner_addr = get_owner_addr(&app);

    let addr = app
        .instantiate_contract(
            code_id,
            owner_addr.clone(),
            &InstantiateMsg { owner: owner_addr.to_string() },
            &[],
            "orbital-core",
            None,
        )
        .unwrap();

    let resp: Ownership<String> = app
        .wrap()
        .query_wasm_smart(addr, &QueryMsg::Ownership {})
        .unwrap();

    assert_eq!(resp.owner, Some(owner_addr.to_string()));
}