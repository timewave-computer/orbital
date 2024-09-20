use cosmwasm_std::{coins, Uint64};
use cw_multi_test::Executor;
use cw_ownable::Ownership;

use orbital_core::{
    msg::{ExecuteMsg, QueryMsg},
    orbital_domain::UncheckedOrbitalDomainConfig,
    state::{OrbitalDomainConfig, UserConfig},
};

use crate::{
    testing_utils::consts::{DENOM_NTRN, GAIA_DOMAIN, OSMOSIS_DOMAIN, USER_1},
    tests::test_orbital_core::suite::OrbitalCoreBuilder,
};

#[test]
fn test_init() {
    let suite = OrbitalCoreBuilder::default().build();

    let resp: Ownership<String> = suite
        .app
        .wrap()
        .query_wasm_smart(suite.orbital_core, &QueryMsg::Ownership {})
        .unwrap();

    assert_eq!(resp.owner, Some(suite.owner.to_string()));
}

#[test]
#[should_panic(expected = "Invalid input")]
fn test_register_orbital_domain_validates_addr() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite
        .register_new_domain(
            GAIA_DOMAIN,
            UncheckedOrbitalDomainConfig::Polytone {
                note: "invalid_note".to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "Orbital domain already registered: ")]
fn test_register_duplicate_orbital_domain() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite
        .register_new_domain(
            GAIA_DOMAIN,
            UncheckedOrbitalDomainConfig::Polytone {
                note: suite.note.to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();
    suite
        .register_new_domain(
            GAIA_DOMAIN,
            UncheckedOrbitalDomainConfig::Polytone {
                note: suite.note.to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "Caller is not the contract's current owner")]
fn test_register_orbital_domain_validates_domain_owner() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite
        .app
        .execute_contract(
            suite.note.clone(),
            suite.orbital_core,
            &ExecuteMsg::RegisterNewDomain {
                domain: GAIA_DOMAIN.to_string(),
                account_type: UncheckedOrbitalDomainConfig::Polytone {
                    note: suite.note.to_string(),
                    timeout: Uint64::one(),
                },
            },
            &[],
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "timeout must be non-zero")]
fn test_register_orbital_ica_domain_validates_timeout() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite
        .register_new_domain(
            GAIA_DOMAIN,
            UncheckedOrbitalDomainConfig::InterchainAccount {
                connection_id: "connection-id".to_string(),
                channel_id: "channel-id".to_string(),
                timeout: Uint64::zero(),
            },
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "timeout must be non-zero")]
fn test_register_orbital_polytone_domain_validates_timeout() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite
        .register_new_domain(
            OSMOSIS_DOMAIN,
            UncheckedOrbitalDomainConfig::Polytone {
                note: suite.note.to_string(),
                timeout: Uint64::zero(),
            },
        )
        .unwrap();
}

#[test]
fn test_register_orbital_domain_happy() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite
        .register_new_domain(
            OSMOSIS_DOMAIN,
            UncheckedOrbitalDomainConfig::Polytone {
                note: suite.note.to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();

    suite
        .register_new_domain(
            GAIA_DOMAIN,
            UncheckedOrbitalDomainConfig::InterchainAccount {
                connection_id: "connection-id".to_string(),
                channel_id: "channel-id".to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();

    let polytone_domain = suite.query_domain(OSMOSIS_DOMAIN).unwrap();
    let ica_domain = suite.query_domain(GAIA_DOMAIN).unwrap();

    assert!(
        polytone_domain
            == OrbitalDomainConfig::Polytone {
                note: suite.note,
                timeout: Uint64::one()
            }
    );

    assert!(
        ica_domain
            == OrbitalDomainConfig::InterchainAccount {
                connection_id: "connection-id".to_string(),
                channel_id: "channel-id".to_string(),
                timeout: Uint64::one()
            }
    );
}

#[test]
#[should_panic(expected = "User already registered")]
fn test_register_user_duplicate() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite.register_user(USER_1).unwrap();
    suite.register_user(USER_1).unwrap();
}

#[test]
fn test_register_user_happy() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite.register_user(USER_1).unwrap();

    let user_config = suite.query_user(USER_1).unwrap();

    assert!(user_config == UserConfig::default());
}

#[test]
#[should_panic(expected = "User not registered")]
fn test_register_user_new_domain_validates_user_registration() {
    let mut suite = OrbitalCoreBuilder::default().build();

    suite
        .register_user_to_new_domain(USER_1, GAIA_DOMAIN, vec![])
        .unwrap();
}

#[test]
#[should_panic(expected = "Unknown domain: gaia")]
fn test_register_user_new_domain_validates_domain_existance() {
    let mut suite = OrbitalCoreBuilder::default().build();
    suite.register_user(USER_1).unwrap();

    suite
        .register_user_to_new_domain(USER_1, GAIA_DOMAIN, vec![])
        .unwrap();
}

#[test]
#[should_panic(expected = "Domain registration error: No funds sent")]
fn test_register_user_new_ica_domain_asserts_fee_denom() {
    let mut suite = OrbitalCoreBuilder::default().build();
    suite.register_user(USER_1).unwrap();
    suite
        .register_new_domain(
            GAIA_DOMAIN,
            UncheckedOrbitalDomainConfig::InterchainAccount {
                connection_id: "connection-id".to_string(),
                channel_id: "channel-id".to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();

    suite
        .register_user_to_new_domain(USER_1, GAIA_DOMAIN, vec![])
        .unwrap();
}

#[test]
#[should_panic(expected = "Domain registration error: insufficient fee")]
fn test_register_user_new_ica_domain_asserts_insufficient_fee() {
    let mut suite = OrbitalCoreBuilder::default().build();
    suite.register_user(USER_1).unwrap();
    suite
        .register_new_domain(
            GAIA_DOMAIN,
            UncheckedOrbitalDomainConfig::InterchainAccount {
                connection_id: "connection-id".to_string(),
                channel_id: "channel-id".to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();

    suite
        .register_user_to_new_domain(USER_1, GAIA_DOMAIN, coins(1, DENOM_NTRN))
        .unwrap();
}

#[test]
fn test_register_user_new_ica_domain_happy() {
    let mut suite = OrbitalCoreBuilder::default().build();
    suite.register_user(USER_1).unwrap();
    suite
        .register_new_domain(
            GAIA_DOMAIN,
            UncheckedOrbitalDomainConfig::InterchainAccount {
                connection_id: "connection-id".to_string(),
                channel_id: "channel-id".to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();

    suite
        .register_user_to_new_domain(USER_1, GAIA_DOMAIN, coins(1, DENOM_NTRN))
        .unwrap();

    let user_domains = suite.query_user_domains(USER_1).unwrap();
    let clearing_account = suite.query_clearing_account(GAIA_DOMAIN, USER_1).unwrap();

    assert_eq!(user_domains.len(), 1);
    assert_eq!(user_domains.first(), Some(&GAIA_DOMAIN.to_string()));
    assert_eq!(clearing_account, None);
}
