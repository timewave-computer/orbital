use cosmwasm_std::{coin, Addr, Coin, StdResult, Uint128};
use cw_multi_test::{error::AnyResult, AppResponse, Executor};
use cw_utils::Duration;
use orbital_auction::{
    msg::{ExecuteMsg, InstantiateMsg as OrbitalAuctionInstantiateMsg, QueryMsg},
    state::{AuctionConfig, AuctionPhase, AuctionRound, RouteConfig, UserIntent},
};

use crate::testing_utils::{
    base_suite_builder::SuiteBuilder,
    consts::{DENOM_ATOM, DENOM_OSMO, GAIA_DOMAIN, OSMOSIS_DOMAIN},
    types::CustomApp,
};

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
                    src_domain: GAIA_DOMAIN.to_string(),
                    dest_domain: OSMOSIS_DOMAIN.to_string(),
                    offer_denom: DENOM_ATOM.to_string(),
                    ask_denom: DENOM_OSMO.to_string(),
                },
                batch_size: Uint128::new(10_000_000),
                auction_duration: Duration::Time(180),
                filling_window_duration: Duration::Time(60),
                cleanup_window_duration: Duration::Time(60),
                solver_bond: coin(100_000, DENOM_ATOM),
            },
        }
    }
}

pub fn user_intent_1() -> UserIntent {
    UserIntent {
        user: "user1".to_string(),
        amount: Uint128::new(100),
        offer_domain: GAIA_DOMAIN.to_string(),
        ask_domain: OSMOSIS_DOMAIN.to_string(),
    }
}

pub fn user_intent_2() -> UserIntent {
    UserIntent {
        user: "user2".to_string(),
        amount: Uint128::new(321),
        offer_domain: GAIA_DOMAIN.to_string(),
        ask_domain: OSMOSIS_DOMAIN.to_string(),
    }
}

impl Suite {
    pub fn sync(&mut self) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            self.orbital_core.clone(),
            self.orbital_auction.clone(),
            &ExecuteMsg::Tick {},
            &[],
        )
    }

    pub fn add_order(&mut self, user_intent: UserIntent) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            self.orbital_core.clone(),
            self.orbital_auction.clone(),
            &ExecuteMsg::AddOrder(user_intent),
            &[],
        )
    }

    pub fn post_bond(&mut self, solver: Addr, bond: Coin) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            solver,
            self.orbital_auction.clone(),
            &ExecuteMsg::PostBond {},
            &[bond],
        )
    }

    pub fn withdraw_bond(&mut self, solver: Addr) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            solver,
            self.orbital_auction.clone(),
            &ExecuteMsg::WithdrawBond {},
            &[],
        )
    }

    pub fn advance_time(&mut self, seconds: u64) {
        self.app
            .update_block(|b| b.time = b.time.plus_seconds(seconds));
    }

    pub fn advance_to_next_phase(&mut self) {
        let active_round_phases = self.query_active_round_config().unwrap().phases;
        let current_phase = self.query_current_phase().unwrap();

        let target_expiration = match current_phase {
            AuctionPhase::Bidding => active_round_phases.auction_expiration,
            AuctionPhase::Filling => active_round_phases.filling_expiration,
            AuctionPhase::Cleanup => active_round_phases.cleanup_expiration,
            AuctionPhase::OutOfSync => {
                let new_exp = active_round_phases.cleanup_expiration + Duration::Time(1);
                new_exp.unwrap()
            }
        };

        let target_time = match target_expiration {
            cw_utils::Expiration::AtTime(timestamp) => timestamp,
            _ => panic!(),
        };
        self.app.update_block(|b| b.time = target_time);
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

    pub fn query_active_round_config(&mut self) -> StdResult<AuctionRound> {
        self.app
            .wrap()
            .query_wasm_smart(self.orbital_auction.clone(), &QueryMsg::ActiveRound {})
    }

    pub fn query_current_phase(&mut self) -> StdResult<AuctionPhase> {
        self.app
            .wrap()
            .query_wasm_smart(self.orbital_auction.clone(), &QueryMsg::AuctionPhase {})
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

    pub fn query_posted_bond(&mut self, solver: &str) -> StdResult<Coin> {
        self.app.wrap().query_wasm_smart(
            self.orbital_auction.clone(),
            &QueryMsg::PostedBond {
                solver: solver.to_string(),
            },
        )
    }
}

impl OrbitalAuctionBuilder {
    pub fn build(mut self) -> Suite {
        let orbital_core = self.builder.admin.clone();
        let solver = self.builder.solver.clone();

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
            solver,
        }
    }
}

pub struct Suite {
    pub app: CustomApp,
    pub orbital_core: Addr,
    pub orbital_auction: Addr,
    pub solver: Addr,
}
