use cosmwasm_std::Uint128;
use cw_utils::Duration;
use orbital_auction::state::{RouteConfig, UserIntent};

use super::suite::OrbitalAuctionBuilder;

#[test]
fn test_init() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    let admin = suite.query_admin().unwrap();
    let auction_config = suite.query_auction_config().unwrap();

    assert_eq!(admin, suite.orbital_core);
    assert_eq!(auction_config.batch_size.u128(), 10_000_000);
    assert_eq!(auction_config.auction_duration, Duration::Time(180));
    assert_eq!(auction_config.filling_window_duration, Duration::Time(60));
    assert_eq!(
        auction_config.route_config,
        RouteConfig {
            src_domain: "gaia".to_string(),
            dest_domain: "juno".to_string(),
            offer_denom: "uatom".to_string(),
            ask_denom: "ujuno".to_string(),
        }
    );
}

#[test]
fn test_add_user_intents() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    let user_intent_1 = UserIntent {
        user: "user1".to_string(),
        amount: Uint128::new(100),
        offer_domain: "gaia".to_string(),
        ask_domain: "juno".to_string(),
    };
    let user_intent_2 = UserIntent {
        user: "user2".to_string(),
        amount: Uint128::new(321),
        offer_domain: "gaia".to_string(),
        ask_domain: "juno".to_string(),
    };

    // add the user intents, order matters here
    suite.add_order(user_intent_1).unwrap();
    suite.add_order(user_intent_2).unwrap();

    let orderbook = suite.query_orderbook().unwrap();
    assert_eq!(orderbook.len(), 2);
    assert_eq!(orderbook[0].user, "user1");
    assert_eq!(orderbook[0].amount, Uint128::new(100));
    assert_eq!(orderbook[1].user, "user2");
    assert_eq!(orderbook[1].amount, Uint128::new(321));
}
