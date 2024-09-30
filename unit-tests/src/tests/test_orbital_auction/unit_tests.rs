use cw_utils::Duration;
use orbital_auction::state::RouteConfig;

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
