use cosmwasm_std::{coin, Uint64};
use localic_std::{
    errors::LocalError, modules::cosmwasm::contract_execute, types::TransactionResponse,
};
use localic_utils::{utils::test_context::TestContext, NEUTRON_CHAIN_NAME};
use log::info;
use orbital_core::{msg::ExecuteMsg, orbital_domain::UncheckedOrbitalDomainConfig};

use crate::ACC0_KEY;

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

pub fn register_icq_balances_query(
    test_ctx: &TestContext,
    orbital_core: String,
    domain: String,
    addr: String,
    denoms: Vec<String>,
) -> Result<TransactionResponse, LocalError> {
    info!("registering ICQ balances query on domain {domain} for {addr}...");

    let register_icq_msg = ExecuteMsg::RegisterBalancesQuery {
        connection_id: test_ctx
            .get_connections()
            .src(NEUTRON_CHAIN_NAME)
            .dest(&domain)
            .get(),
        update_period: 5,
        addr,
        denoms,
    };

    contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        ACC0_KEY,
        &serde_json::to_string(&register_icq_msg)
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
        "--amount 10000000untrn --gas 50000000",
    )
}

pub fn register_icq_transfers_query(
    test_ctx: &TestContext,
    orbital_core: String,
    domain: String,
    addr: String,
) -> Result<TransactionResponse, LocalError> {
    info!("registering ICQ transfers query on domain {domain} for {addr}...");

    let register_icq_msg = ExecuteMsg::RegisterTransfersQuery {
        connection_id: test_ctx
            .get_connections()
            .src(NEUTRON_CHAIN_NAME)
            .dest(&domain)
            .get(),
        update_period: 5,
        recipient: addr,
        min_height: None,
    };

    contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        ACC0_KEY,
        &serde_json::to_string(&register_icq_msg)
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
        "--amount 10000000untrn --gas 50000000",
    )
}

pub fn user_withdraw_funds_from_domain(
    test_ctx: &TestContext,
    orbital_core: String,
    user_key: &str,
    domain: String,
    addr: String,
    amount: u128,
    denom: &str,
) -> Result<TransactionResponse, LocalError> {
    info!("user {user_key} withdraw request on {domain} for {amount}{denom} to {addr}...");

    let withdraw_funds_msg = ExecuteMsg::UserWithdrawFunds {
        domain,
        coin: coin(amount, denom),
        dest: addr,
    };

    contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        user_key,
        &serde_json::to_string(&withdraw_funds_msg)
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
        "--amount 10000000untrn --gas 50000000",
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
