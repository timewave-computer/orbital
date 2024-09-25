use cosmwasm_std::Uint64;
use localic_std::{
    errors::LocalError,
    modules::cosmwasm::{contract_execute, contract_query},
    types::TransactionResponse,
};
use localic_utils::{utils::test_context::TestContext, NEUTRON_CHAIN_NAME};
use log::info;
use orbital_core::{
    msg::{ExecuteMsg, QueryMsg},
    orbital_domain::UncheckedOrbitalDomainConfig,
    state::UserConfig,
};

use crate::ACC0_KEY;

pub fn query_user_clearing_acc_addr_on_domain(
    test_ctx: &TestContext,
    orbital_core: String,
    user_addr: &str,
    domain: String,
) -> Result<Option<String>, LocalError> {
    let clearing_acc_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        &serde_json::to_string(&QueryMsg::ClearingAccountAddress {
            addr: user_addr.to_string(),
            domain: domain.to_string(),
        })
        .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
    )["data"]
        .clone();
    let user_clearing_acc: Option<String> = serde_json::from_value(clearing_acc_response)
        .map_err(|e| LocalError::Custom { msg: e.to_string() })?;

    info!(
        "user {user_addr} clearing account on {domain}: {:?}",
        user_clearing_acc
    );

    Ok(user_clearing_acc)
}

pub fn query_user_config(
    test_ctx: &TestContext,
    orbital_core: String,
    user_addr: &str,
) -> Result<UserConfig, LocalError> {
    let query_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        &serde_json::to_string(&QueryMsg::UserConfig {
            addr: user_addr.to_string(),
        })
        .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
    )["data"]
        .clone();

    let user_config: UserConfig = serde_json::from_value(query_response).unwrap();

    info!("user {user_addr} config: {:?}", user_config);

    Ok(user_config)
}

pub fn user_register_orbital_core(
    test_ctx: &TestContext,
    user_key: &str,
    orbital_core: String,
) -> Result<TransactionResponse, LocalError> {
    info!("registering user {user_key} to orbital-core...");
    contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        user_key,
        &serde_json::to_string(&ExecuteMsg::RegisterUser {})
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
        "",
    )
}

pub fn user_register_to_new_domain(
    test_ctx: &TestContext,
    user_key: &str,
    orbital_core: String,
    domain: String,
) -> Result<TransactionResponse, LocalError> {
    info!("registering user {user_key} to {domain} domain...");
    contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        user_key,
        &serde_json::to_string(&ExecuteMsg::RegisterUserDomain { domain })
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
        "--amount 1000000untrn --gas 5000000",
    )
}

pub fn admin_register_domain(
    test_ctx: &TestContext,
    orbital_core: String,
    domain: String,
) -> Result<TransactionResponse, LocalError> {
    let admin_register_domain_msg = ExecuteMsg::RegisterNewDomain {
        domain: domain.to_string(),
        account_type: UncheckedOrbitalDomainConfig::InterchainAccount {
            connection_id: test_ctx
                .get_connections()
                .src(NEUTRON_CHAIN_NAME)
                .dest(&domain)
                .get(),
            channel_id: test_ctx
                .get_transfer_channels()
                .src(NEUTRON_CHAIN_NAME)
                .dest(&domain)
                .get(),
            timeout: Uint64::new(100),
        },
    };

    info!("admin registering orbital-level {domain} domain...");
    contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        ACC0_KEY,
        &serde_json::to_string(&admin_register_domain_msg)
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
        "",
    )
}
