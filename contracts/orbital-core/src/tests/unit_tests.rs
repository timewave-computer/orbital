use cosmwasm_std::Uint64;
use cw_multi_test::Executor;
use cw_ownable::Ownership;

use crate::{
    domain::UncheckedDomainConfig,
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
            &ExecuteMsg::RegisterNewDomain(UncheckedDomainConfig::Polytone {
                domain: "domain".to_string(),
                note: "invalid_note".to_string(),
                timeout: Uint64::one(),
            }),
            &[],
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "empty domain")]
fn test_register_orbital_domain_validates_domain() {
    let mut suite = Suite::default();

    suite
        .app
        .execute_contract(
            suite.owner,
            suite.orbital,
            &ExecuteMsg::RegisterNewDomain(UncheckedDomainConfig::Polytone {
                domain: "".to_string(),
                note: suite.note.to_string(),
                timeout: Uint64::one(),
            }),
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
            &ExecuteMsg::RegisterNewDomain(UncheckedDomainConfig::Polytone {
                domain: "domain".to_string(),
                note: suite.note.to_string(),
                timeout: Uint64::one(),
            }),
            &[],
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "timeout must be non-zero")]
fn test_register_orbital_domain_validates_timeout() {
    let mut suite = Suite::default();

    suite
        .app
        .execute_contract(
            suite.owner,
            suite.orbital,
            &ExecuteMsg::RegisterNewDomain(UncheckedDomainConfig::Polytone {
                domain: "domain".to_string(),
                note: suite.note.to_string(),
                timeout: Uint64::zero(),
            }),
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
            suite.owner,
            suite.orbital.clone(),
            &ExecuteMsg::RegisterNewDomain(UncheckedDomainConfig::Polytone {
                domain: "domain".to_string(),
                note: suite.note.to_string(),
                timeout: Uint64::one(),
            }),
            &[],
        )
        .unwrap();

    let domain: OrbitalDomainConfig = suite
        .app
        .wrap()
        .query_wasm_smart(
            suite.orbital.clone(),
            &QueryMsg::OrbitalDomain {
                domain: "domain".to_string(),
            },
        )
        .unwrap();

    match domain {
        OrbitalDomainConfig::Polytone {
            domain,
            note,
            timeout,
        } => {
            assert_eq!(domain, "domain");
            assert_eq!(note, suite.note);
            assert_eq!(timeout, Uint64::one());
        }
    }
}
