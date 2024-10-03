use cosmwasm_std::{coin, Uint128, Uint64};
use cw_utils::{Duration, Expiration};
use orbital_auction::state::{AuctionPhase, RoundPhaseExpirations, RouteConfig, UserIntent};

use crate::{
    testing_utils::consts::{DENOM_ATOM, DENOM_NTRN, DENOM_OSMO, GAIA_DOMAIN, OSMOSIS_DOMAIN},
    tests::test_orbital_auction::suite::{user_intent_1, user_intent_2},
};

use super::suite::OrbitalAuctionBuilder;

#[test]
fn test_init() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    let current_time = suite.app.block_info().time;

    let admin = suite.query_admin().unwrap();
    let auction_config = suite.query_auction_config().unwrap();
    let active_round = suite.query_active_round_config().unwrap();

    assert_eq!(admin, suite.orbital_core);
    assert_eq!(auction_config.batch_size.u128(), 10_000_000);
    assert_eq!(
        auction_config.auction_phases.auction_duration,
        Duration::Time(180)
    );
    assert_eq!(
        auction_config.auction_phases.filling_window_duration,
        Duration::Time(60)
    );
    assert_eq!(
        auction_config.auction_phases.cleanup_window_duration,
        Duration::Time(60)
    );
    assert_eq!(
        auction_config.route_config,
        RouteConfig {
            src_domain: GAIA_DOMAIN.to_string(),
            dest_domain: OSMOSIS_DOMAIN.to_string(),
            offer_denom: DENOM_ATOM.to_string(),
            ask_denom: DENOM_OSMO.to_string(),
        }
    );
    assert_eq!(active_round.id.u64(), 0);
    assert_eq!(active_round.batch.batch_size, Uint128::zero());
    assert_eq!(active_round.batch.batch_capacity, Uint128::new(10000000));
    assert_eq!(
        active_round.phases,
        RoundPhaseExpirations {
            start_expiration: Expiration::AtTime(current_time),
            auction_expiration: Expiration::AtTime(current_time.plus_seconds(180)),
            filling_expiration: Expiration::AtTime(current_time.plus_seconds(180 + 60)),
            cleanup_expiration: Expiration::AtTime(current_time.plus_seconds(180 + 60 + 60)),
        }
    );
}

#[test]
fn test_round_phase_derivation() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    // 0-180 = auction
    assert!(suite.query_current_phase().unwrap() == AuctionPhase::Bidding);
    suite.advance_time(30);
    assert!(suite.query_current_phase().unwrap() == AuctionPhase::Bidding);
    suite.advance_time(149);
    assert!(suite.query_current_phase().unwrap() == AuctionPhase::Bidding);

    // 180-240 = filling
    suite.advance_time(1);
    assert!(suite.query_current_phase().unwrap() == AuctionPhase::Filling);
    suite.advance_time(58);
    assert!(suite.query_current_phase().unwrap() == AuctionPhase::Filling);

    // 240-300 = cleanup
    suite.advance_time(2);
    assert!(suite.query_current_phase().unwrap() == AuctionPhase::Cleanup);
    suite.advance_time(57);
    assert!(suite.query_current_phase().unwrap() == AuctionPhase::Cleanup);

    suite.advance_time(3);
    assert!(suite.query_current_phase().unwrap() == AuctionPhase::OutOfSync);
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

#[test]
#[should_panic(expected = "auction phase error")]
fn test_finalize_round_before_start_phase() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    // advance to filling phase
    suite.advance_to_next_phase();
    // advance to cleanup phase
    suite.advance_to_next_phase();

    suite.tick(true).unwrap();
    suite.tick(true).unwrap();
}

#[test]
#[should_panic(expected = "auction phase error")]
fn test_finalize_round_bidding_phase() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    suite.tick(false).unwrap();
}

#[test]
fn test_finalize_round_filling_phase_filled() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();

    let orderbook = suite.query_orderbook().unwrap();
    assert_eq!(orderbook.len(), 2);
    let active_round = suite.query_active_round_config().unwrap();
    assert_eq!(active_round.id, Uint64::zero());

    suite.tick(true).unwrap();

    let orderbook = suite.query_orderbook().unwrap();
    assert_eq!(orderbook.len(), 0);
    let active_round = suite.query_active_round_config().unwrap();
    assert_eq!(active_round.id, Uint64::new(1));
}

#[test]
fn test_finalize_round_filling_phase_not_filled_noop() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();

    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();
    // advance to cleanup phase
    suite.advance_to_next_phase();

    let orderbook = suite.query_orderbook().unwrap();
    assert_eq!(orderbook.len(), 2);
    let active_round = suite.query_active_round_config().unwrap();
    assert_eq!(active_round.id, Uint64::zero());

    suite.tick(false).unwrap();

    // advance to bidding phase
    suite.advance_to_next_phase();

    suite.bid(solver.clone(), 5_000).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();

    // ticking without fulfilling the bid in the filling phase is a no-op
    suite.tick(false).unwrap();
}

#[test]
fn test_finalize_round_cleanup_phase_filled() {
    let mut suite = OrbitalAuctionBuilder::default().build();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();
    // advance to cleanup phase
    suite.advance_to_next_phase();

    let orderbook = suite.query_orderbook().unwrap();
    assert_eq!(orderbook.len(), 2);
    let active_round = suite.query_active_round_config().unwrap();
    assert_eq!(active_round.id, Uint64::zero());

    suite.tick(true).unwrap();

    // let orderbook = suite.query_orderbook().unwrap();
    // assert_eq!(orderbook.len(), 0);
    let active_round = suite.query_active_round_config().unwrap();
    assert_eq!(active_round.id, Uint64::new(1));
}

#[test]
fn test_finalize_round_cleanup_phase_not_filled_slashes_solver() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();

    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();

    // advance to cleanup phase
    suite.advance_to_next_phase();

    let active_round = suite.query_active_round_config().unwrap();
    assert_eq!(active_round.id, Uint64::zero());
    let solver_bond = suite.query_posted_bond(solver.as_str()).unwrap();
    assert_eq!(solver_bond, coin(100_000, DENOM_ATOM));

    // tick with no bids placed, forcing the next round to prepare
    suite.tick(false).unwrap();

    let active_round = suite.query_active_round_config().unwrap();
    assert_eq!(active_round.id, Uint64::new(1));

    // advance to bidding phase
    suite.advance_to_next_phase();

    // solver places a bid
    suite.bid(solver.clone(), 5_000).unwrap();
    if let Some(bid) = suite.query_active_round_config().unwrap().batch.current_bid {
        assert_eq!(bid.amount, Uint128::new(5_000));
        assert_eq!(bid.solver, solver);
    }

    // advance to filling phase
    suite.advance_to_next_phase();

    let solver_bond = suite.query_posted_bond(solver.as_str()).unwrap();
    assert_eq!(solver_bond, coin(100_000, DENOM_ATOM));

    // advance to cleanup phase
    suite.advance_to_next_phase();

    // tick with no-fill flag, which should slash the solver
    suite.tick(false).unwrap();

    let solver_bond = suite.query_posted_bond(solver.as_str()).unwrap();
    assert_eq!(solver_bond, coin(0, DENOM_ATOM));
}

#[test]
#[should_panic]
fn test_finalize_round_out_of_sync_phase() {
    unimplemented!()
}

#[test]
#[should_panic(expected = "Insufficient bond posted")]
fn test_solver_bid_with_insufficient_bond_amount() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();

    suite
        .post_bond(solver.clone(), coin(10_000, DENOM_ATOM))
        .unwrap();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();

    // advance to cleanup phase
    suite.advance_to_next_phase();

    // tick with no bids placed, forcing the next round to prepare
    suite.tick(false).unwrap();

    // advance to bidding phase
    suite.advance_to_next_phase();

    // solver places a bid
    suite.bid(solver.clone(), 5_000).unwrap();
}

#[test]
#[should_panic(expected = "auction phase error")]
fn test_solver_bid_during_non_bidding_phase() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();

    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();

    // solver places a bid
    suite.bid(solver.clone(), 5_000).unwrap();
}

#[test]
#[should_panic(expected = "Current bid is higher")]
fn test_solver_bid_with_lower_than_current_bid_amount() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();
    let solver_2 = suite.solver_2.clone();

    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();
    suite
        .post_bond(solver_2.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();

    // advance to cleanup phase
    suite.advance_to_next_phase();

    // tick with no bids placed, forcing the next round to prepare
    suite.tick(false).unwrap();

    // advance to bidding phase
    suite.advance_to_next_phase();

    suite.bid(solver.clone(), 5_000).unwrap();
    suite.bid(solver_2.clone(), 4_000).unwrap();
}

#[test]
fn test_solver_bid_happy() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();
    let solver_2 = suite.solver_2.clone();

    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();
    suite
        .post_bond(solver_2.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();

    // advance to cleanup phase
    suite.advance_to_next_phase();

    // tick with no bids placed, forcing the next round to prepare
    suite.tick(false).unwrap();

    // advance to bidding phase
    suite.advance_to_next_phase();

    let current_bid = suite.query_active_round_config().unwrap().batch.current_bid;
    assert_eq!(current_bid, None);

    suite.bid(solver.clone(), 5_000).unwrap();

    let current_bid = suite
        .query_active_round_config()
        .unwrap()
        .batch
        .current_bid
        .unwrap();
    assert_eq!(current_bid.amount.u128(), 5_000);
    assert_eq!(current_bid.solver, solver);

    // solver 2 overbids
    suite.bid(solver_2.clone(), 10_000).unwrap();

    let current_bid = suite
        .query_active_round_config()
        .unwrap()
        .batch
        .current_bid
        .unwrap();
    assert_eq!(current_bid.amount.u128(), 10_000);
    assert_eq!(current_bid.solver, solver_2);
}

#[test]
#[should_panic(expected = "Cannot withdraw bond while being the highest active bidder")]
fn test_solver_withdraw_posted_bond_while_winning_bidder() {
    let mut suite = OrbitalAuctionBuilder::default().build();
    let solver = suite.solver.clone();

    suite
        .post_bond(solver.clone(), coin(100_000, DENOM_ATOM))
        .unwrap();

    // add a couple of user intents to the orderbook
    suite.add_order(user_intent_1()).unwrap();
    suite.add_order(user_intent_2()).unwrap();

    // advance to filling phase
    suite.advance_to_next_phase();

    // advance to cleanup phase
    suite.advance_to_next_phase();

    // tick with no bids placed, forcing the next round to prepare
    suite.tick(false).unwrap();

    // advance to bidding phase
    suite.advance_to_next_phase();

    let current_bid = suite.query_active_round_config().unwrap().batch.current_bid;
    assert_eq!(current_bid, None);

    suite.bid(solver.clone(), 5_000).unwrap();

    let current_bid = suite
        .query_active_round_config()
        .unwrap()
        .batch
        .current_bid
        .unwrap();
    assert_eq!(current_bid.amount.u128(), 5_000);
    assert_eq!(current_bid.solver, solver);

    suite.withdraw_bond(solver.clone()).unwrap();
}
