use cosmwasm_std::{coin, Uint128};
use cw_utils::Duration;
use orbital_auction::state::{RouteConfig, UserIntent};

use crate::testing_utils::consts::{
    DENOM_ATOM, DENOM_NTRN, DENOM_OSMO, GAIA_DOMAIN, OSMOSIS_DOMAIN,
};

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
            src_domain: GAIA_DOMAIN.to_string(),
            dest_domain: OSMOSIS_DOMAIN.to_string(),
            offer_denom: DENOM_ATOM.to_string(),
            ask_denom: DENOM_OSMO.to_string(),
        }
    );
}

#[test]
fn test_add_user_intents() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    let user_intent_1 = UserIntent {
        user: "user1".to_string(),
        amount: Uint128::new(100),
        offer_domain: GAIA_DOMAIN.to_string(),
        ask_domain: OSMOSIS_DOMAIN.to_string(),
    };
    let user_intent_2 = UserIntent {
        user: "user2".to_string(),
        amount: Uint128::new(321),
        offer_domain: GAIA_DOMAIN.to_string(),
        ask_domain: OSMOSIS_DOMAIN.to_string(),
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

#[test]
#[should_panic(expected = "Must send reserve token 'uatom'")]
fn test_solver_posting_wrong_bond_denom() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();

    // solver posts bond with wrong denom
    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_NTRN))
        .unwrap();
}

#[test]
fn test_solver_posting_bond_happy() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();

    // all solvers start with no bond posted
    let posted_bond = suite.query_posted_bond(solver.as_str()).unwrap();
    assert_eq!(posted_bond, coin(0, DENOM_ATOM));

    // solver posts bond
    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    let posted_bond = suite.query_posted_bond(solver.as_str()).unwrap();
    assert_eq!(posted_bond, coin(100_000, DENOM_ATOM));

    // for whatever reason solver needs to post more bond
    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    let posted_bond = suite.query_posted_bond(solver.as_str()).unwrap();
    assert_eq!(posted_bond, coin(200_000, DENOM_ATOM));
}

#[test]
fn test_solver_withdraw_posted_bond() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();

    // solver posts bond
    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    let posted_bond = suite.query_posted_bond(solver.as_str()).unwrap();
    assert_eq!(posted_bond, coin(100_000, DENOM_ATOM));

    let current_solver_atom_bal = suite
        .app
        .wrap()
        .query_balance(solver.to_string(), DENOM_ATOM)
        .unwrap();
    let current_auction_atom_bal = suite
        .app
        .wrap()
        .query_balance(suite.orbital_auction.to_string(), DENOM_ATOM)
        .unwrap();

    assert_eq!(current_auction_atom_bal.amount.u128(), 100_000);

    // solver is done and withdraws bond
    suite.withdraw_bond(solver.clone()).unwrap();

    let posted_bond = suite.query_posted_bond(solver.as_str()).unwrap();
    assert_eq!(posted_bond, coin(0, DENOM_ATOM));

    let new_solver_atom_bal = suite
        .app
        .wrap()
        .query_balance(solver.to_string(), DENOM_ATOM)
        .unwrap();
    let new_auction_atom_bal = suite
        .app
        .wrap()
        .query_balance(suite.orbital_auction.to_string(), DENOM_ATOM)
        .unwrap();

    assert_eq!(new_auction_atom_bal.amount.u128(), 0);
    assert_eq!(
        new_solver_atom_bal.amount.u128(),
        current_solver_atom_bal.amount.u128() + 100_000
    );
}
