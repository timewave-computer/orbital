use cosmwasm_std::{Addr, Empty, StdResult, Uint64};
use cw_multi_test::{error::AnyResult, App, AppResponse, Contract, ContractWrapper, Executor};

use crate::{
    account_types::AccountConfigType,
    contract::{execute, instantiate, query},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::OrbitalDomainConfig,
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

impl Suite {
    pub fn query_domain(&mut self, domain: &str) -> StdResult<OrbitalDomainConfig> {
        self.app.wrap().query_wasm_smart(
            self.orbital.clone(),
            &QueryMsg::OrbitalDomain {
                domain: domain.to_string(),
            },
        )
    }
    pub fn register_new_domain(
        &mut self,
        domain: &str,
        account_type: AccountConfigType,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            self.owner.clone(),
            self.orbital.clone(),
            &ExecuteMsg::RegisterNewDomain {
                domain: domain.to_string(),
                account_type,
            },
            &[],
        )
    }
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
