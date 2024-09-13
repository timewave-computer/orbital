use cosmwasm_std::Uint64;
use cw_multi_test::Executor;
use cw_ownable::Ownership;

use crate::{
    account_types::AccountConfigType,
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
        .app
        .execute_contract(
            suite.owner,
            suite.orbital,
            &ExecuteMsg::RegisterNewDomain {
                domain: "domain".to_string(),
                account_type: AccountConfigType::Polytone {
                    note: "invalid_note".to_string(),
                    timeout: Uint64::one(),
                },
            },
            &[],
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "Orbital domain already registered: ")]
fn test_register_duplicate_orbital_domain() {
    let mut suite = Suite::default();

    suite
        .app
        .execute_contract(
            suite.owner.clone(),
            suite.orbital.clone(),
            &ExecuteMsg::RegisterNewDomain {
                domain: "".to_string(),
                account_type: AccountConfigType::Polytone {
                    note: suite.note.to_string(),
                    timeout: Uint64::one(),
                },
            },
            &[],
        )
        .unwrap();

    suite
        .app
        .execute_contract(
            suite.owner,
            suite.orbital,
            &ExecuteMsg::RegisterNewDomain {
                domain: "".to_string(),
                account_type: AccountConfigType::Polytone {
                    note: suite.note.to_string(),
                    timeout: Uint64::one(),
                },
            },
            &[],
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
                account_type: AccountConfigType::Polytone {
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
        .app
        .execute_contract(
            suite.owner,
            suite.orbital,
            &ExecuteMsg::RegisterNewDomain {
                domain: "domain".to_string(),
                account_type: AccountConfigType::InterchainAccount {
                    connection_id: "connection-id".to_string(),
                    channel_id: "channel-id".to_string(),
                    timeout: Uint64::zero(),
                },
            },
            &[],
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "timeout must be non-zero")]
fn test_register_orbital_polytone_domain_validates_timeout() {
    let mut suite = Suite::default();

    suite
        .app
        .execute_contract(
            suite.owner,
            suite.orbital,
            &ExecuteMsg::RegisterNewDomain {
                domain: "domain".to_string(),
                account_type: AccountConfigType::Polytone {
                    note: suite.note.to_string(),
                    timeout: Uint64::zero(),
                },
            },
            &[],
        )
        .unwrap();
}

#[test]
fn test_register_orbital_domain_happy() {
    let mut suite = Suite::default();

    suite
        .app
        .execute_contract(
            suite.owner.clone(),
            suite.orbital.clone(),
            &ExecuteMsg::RegisterNewDomain {
                domain: "domain_polytone".to_string(),
                account_type: AccountConfigType::Polytone {
                    note: suite.note.to_string(),
                    timeout: Uint64::one(),
                },
            },
            &[],
        )
        .unwrap();

    suite
        .app
        .execute_contract(
            suite.owner,
            suite.orbital.clone(),
            &ExecuteMsg::RegisterNewDomain {
                domain: "domain_ica".to_string(),
                account_type: AccountConfigType::InterchainAccount {
                    connection_id: "connection-id".to_string(),
                    channel_id: "channel-id".to_string(),
                    timeout: Uint64::one(),
                },
            },
            &[],
        )
        .unwrap();

    let polytone_domain: OrbitalDomainConfig = suite
        .app
        .wrap()
        .query_wasm_smart(
            suite.orbital.clone(),
            &QueryMsg::OrbitalDomain {
                domain: "domain_polytone".to_string(),
            },
        )
        .unwrap();
    let ica_domain: OrbitalDomainConfig = suite
        .app
        .wrap()
        .query_wasm_smart(
            suite.orbital.clone(),
            &QueryMsg::OrbitalDomain {
                domain: "domain_ica".to_string(),
            },
        )
        .unwrap();

    assert!(matches!(
        polytone_domain,
        OrbitalDomainConfig::Polytone { note, timeout }
        if note == suite.note && timeout == Uint64::one()
    ));

    assert!(matches!(
        ica_domain,
        OrbitalDomainConfig::ICA { connection_id, channel_id, timeout }
        if connection_id == "connection-id" && channel_id == "channel-id" && timeout == Uint64::one()
    ));
}
