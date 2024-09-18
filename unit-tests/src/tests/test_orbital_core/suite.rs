use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{error::AnyResult, AppResponse, Executor};
use orbital_core::{
    account_types::UncheckedOrbitalDomainConfig,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{OrbitalDomainConfig, UserConfig},
};

use crate::testing_utils::{
    base_suite_builder::{make_addr, SuiteBuilder},
    types::CustomApp,
};

pub struct OrbitalCoreBuilder {
    pub builder: SuiteBuilder,
    pub instantiate_msg: InstantiateMsg,
}

impl Default for OrbitalCoreBuilder {
    fn default() -> Self {
        let builder = SuiteBuilder::default();

        let owner = builder.admin.to_string();

        Self {
            builder,
            instantiate_msg: InstantiateMsg { owner },
        }
    }
}

impl OrbitalCoreBuilder {
    pub fn build(mut self) -> Suite {
        let owner = self.builder.admin.clone();
        let note = self.builder.note.clone();
        let user = self.builder.user_addr.clone();

        let orbital_core_addr = self
            .builder
            .app
            .instantiate_contract(
                self.builder.orbital_core_code_id,
                owner.clone(),
                &self.instantiate_msg,
                &[],
                "orbital-core",
                None,
            )
            .unwrap();

        Suite {
            app: self.builder.build(),
            note,
            owner,
            orbital_core: orbital_core_addr,
            user_addr: user,
        }
    }
}

pub struct Suite {
    pub app: CustomApp,
    pub owner: Addr,
    pub orbital_core: Addr,
    pub note: Addr,
    pub user_addr: Addr,
}

impl Suite {
    pub fn register_user(&mut self, user_addr: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            make_addr(&self.app, user_addr),
            self.orbital_core.clone(),
            &ExecuteMsg::RegisterUser {},
            &[],
        )
    }

    pub fn register_user_to_new_domain(
        &mut self,
        user_addr: &str,
        domain: &str,
        funds: Vec<Coin>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            make_addr(&self.app, user_addr),
            self.orbital_core.clone(),
            &ExecuteMsg::RegisterUserDomain {
                domain: domain.to_string(),
            },
            &funds,
        )
    }

    pub fn query_domain(&mut self, domain: &str) -> StdResult<OrbitalDomainConfig> {
        self.app.wrap().query_wasm_smart(
            self.orbital_core.clone(),
            &QueryMsg::OrbitalDomain {
                domain: domain.to_string(),
            },
        )
    }

    pub fn query_user(&mut self, user: &str) -> StdResult<UserConfig> {
        self.app.wrap().query_wasm_smart(
            self.orbital_core.clone(),
            &QueryMsg::UserConfig {
                user: make_addr(&self.app, user).to_string(),
            },
        )
    }

    pub fn register_new_domain(
        &mut self,
        domain: &str,
        account_type: UncheckedOrbitalDomainConfig,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            self.owner.clone(),
            self.orbital_core.clone(),
            &ExecuteMsg::RegisterNewDomain {
                domain: domain.to_string(),
                account_type,
            },
            &[],
        )
    }
}
