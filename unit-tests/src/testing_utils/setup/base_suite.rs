use cosmwasm_std::{Addr, Coin, Empty, MemoryStorage};
use cw_multi_test::{
    App, BankKeeper, FailingModule, GovFailingModule, IbcFailingModule, MockApiBech32,
    StargateAccepting, WasmKeeper,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

use crate::testing_utils::neutron_adapters::custom_module::NeutronKeeper;

pub type CustomApp = App<
    BankKeeper,
    MockApiBech32,
    MemoryStorage,
    NeutronKeeper,
    WasmKeeper<NeutronMsg, NeutronQuery>,
    FailingModule<Empty, Empty, Empty>,
    FailingModule<Empty, Empty, Empty>,
    IbcFailingModule,
    GovFailingModule,
    StargateAccepting,
>;

pub trait BaseSuite {
    fn get_app(&self) -> &CustomApp;

    fn query_balance(&self, addr: &Addr, denom: &str) -> Coin {
        let app = self.get_app();
        app.wrap().query_balance(addr, denom).unwrap()
    }

    fn query_all_balances(&self, addr: &Addr) -> Vec<Coin> {
        let app = self.get_app();
        app.wrap().query_all_balances(addr).unwrap()
    }

    fn assert_balance(&self, addr: impl Into<String>, coin: Coin) {
        let app = self.get_app();
        let bal = app.wrap().query_balance(addr, &coin.denom).unwrap();
        assert_eq!(bal, coin);
    }
}
