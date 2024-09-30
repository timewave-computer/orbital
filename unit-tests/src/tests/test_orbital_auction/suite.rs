use cosmwasm_std::{Addr, StdResult, Uint128};
use cw_multi_test::{error::AnyResult, AppResponse, Executor};
use cw_utils::Duration;
use orbital_auction::{
    msg::{ExecuteMsg, InstantiateMsg as OrbitalAuctionInstantiateMsg, QueryMsg},
    state::{AuctionConfig, RouteConfig, UserIntent},
};

use crate::testing_utils::{base_suite_builder::SuiteBuilder, types::CustomApp};

pub struct OrbitalAuctionBuilder {
    pub builder: SuiteBuilder,
    pub instantiate_msg: OrbitalAuctionInstantiateMsg,
}

impl Default for OrbitalAuctionBuilder {
    fn default() -> Self {
        let builder = SuiteBuilder::default();

        Self {
            builder,
            instantiate_msg: OrbitalAuctionInstantiateMsg {
                route_config: RouteConfig {
                    src_domain: "gaia".to_string(),
                    dest_domain: "juno".to_string(),
                    offer_denom: "uatom".to_string(),
                    ask_denom: "ujuno".to_string(),
                },
                batch_size: Uint128::new(10_000_000),
                auction_duration: Duration::Time(180),
                filling_window_duration: Duration::Time(60),
            },
        }
    }
}

impl Suite {
    pub fn add_order(&mut self, user_intent: UserIntent) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            self.orbital_core.clone(),
            self.orbital_auction.clone(),
            &ExecuteMsg::AddOrder(user_intent),
            &[],
        )
    }
}

impl Suite {
    pub fn query_admin(&mut self) -> StdResult<Addr> {
        self.app
            .wrap()
            .query_wasm_smart(self.orbital_auction.clone(), &QueryMsg::Admin {})
    }

    pub fn query_auction_config(&mut self) -> StdResult<AuctionConfig> {
        self.app
            .wrap()
            .query_wasm_smart(self.orbital_auction.clone(), &QueryMsg::AuctionConfig {})
    }

    pub fn query_orderbook(&mut self) -> StdResult<Vec<UserIntent>> {
        self.app.wrap().query_wasm_smart(
            self.orbital_auction.clone(),
            &QueryMsg::Orderbook {
                from: None,
                limit: None,
            },
        )
    }
}

impl OrbitalAuctionBuilder {
    pub fn build(mut self) -> Suite {
        let orbital_core = self.builder.admin.clone();

        let orbital_auction_addr = self
            .builder
            .app
            .instantiate_contract(
                self.builder.orbital_auction_code_id,
                orbital_core.clone(),
                &self.instantiate_msg,
                &[],
                "orbital-auction",
                None,
            )
            .unwrap();

        Suite {
            app: self.builder.build(),
            orbital_core,
            orbital_auction: orbital_auction_addr,
        }
    }
}

pub struct Suite {
    pub app: CustomApp,
    pub orbital_core: Addr,
    pub orbital_auction: Addr,
}
