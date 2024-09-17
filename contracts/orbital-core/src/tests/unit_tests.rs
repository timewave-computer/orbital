use cosmwasm_std::Uint64;
use cw_multi_test::Executor;
use cw_ownable::Ownership;

use crate::{
    account_types::UncheckedOrbitalDomainConfig,
    msg::{ExecuteMsg, QueryMsg},
    state::OrbitalDomainConfig,
    tests::ctx::Suite,
};

#[test]
fn test_init() {
    let suite = Suite::default();

    let resp: Ownership<String> = suite
        .app
        .wrap()
        .query_wasm_smart(suite.orbital, &QueryMsg::Ownership {})
        .unwrap();

    assert_eq!(resp.owner, Some(suite.owner.to_string()));
}

#[test]
#[should_panic(expected = "Error decoding bech32")]
fn test_register_orbital_domain_validates_addr() {
    let mut suite = Suite::default();

    suite
        .register_new_domain(
            "domain",
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
    let mut suite = Suite::default();

    suite
        .register_new_domain(
            "",
            UncheckedOrbitalDomainConfig::Polytone {
                note: suite.note.to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();
    suite
        .register_new_domain(
            "",
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
    let mut suite = Suite::default();

    suite
        .app
        .execute_contract(
            suite.note.clone(),
            suite.orbital,
            &ExecuteMsg::RegisterNewDomain {
                domain: "domain".to_string(),
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
    let mut suite = Suite::default();

    suite
        .register_new_domain(
            "domain",
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
    let mut suite = Suite::default();

    suite
        .register_new_domain(
            "domain",
            UncheckedOrbitalDomainConfig::Polytone {
                note: suite.note.to_string(),
                timeout: Uint64::zero(),
            },
        )
        .unwrap();
}

#[test]
fn test_register_orbital_domain_happy() {
    let mut suite = Suite::default();

    suite
        .register_new_domain(
            "domain_polytone",
            UncheckedOrbitalDomainConfig::Polytone {
                note: suite.note.to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();

    suite
        .register_new_domain(
            "domain_ica",
            UncheckedOrbitalDomainConfig::InterchainAccount {
                connection_id: "connection-id".to_string(),
                channel_id: "channel-id".to_string(),
                timeout: Uint64::one(),
            },
        )
        .unwrap();

    let polytone_domain = suite.query_domain("domain_polytone").unwrap();
    let ica_domain = suite.query_domain("domain_ica").unwrap();

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
    let mut suite = Suite::default();

    suite.register_user("user").unwrap();
    suite.register_user("user").unwrap();
}

#[test]
fn test_register_user_happy() {
    let mut suite = Suite::default();

    suite.register_user("user").unwrap();

    let user_config = suite.query_user("user").unwrap();

    assert!(user_config.clearing_accounts.is_empty());
}

#[test]
#[should_panic(expected = "User not registered")]
fn test_register_user_new_domain_validates_user_registration() {
    unimplemented!()
}

#[test]
#[should_panic(expected = "Unknown domain: gaia")]
fn test_register_user_new_domain_validates_domain_existance() {
    unimplemented!()
}
