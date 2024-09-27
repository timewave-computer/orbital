use localic_std::{errors::LocalError, modules::cosmwasm::contract_query};
use localic_utils::{utils::test_context::TestContext, NEUTRON_CHAIN_NAME};
use log::info;
use orbital_core::{
    msg::{GetTransfersAmountResponse, QueryMsg, RecipientTxsResponse},
    state::{ClearingAccountConfig, UserConfig},
};

pub fn query_user_clearing_acc_addr_on_domain(
    test_ctx: &TestContext,
    orbital_core: String,
    user_addr: &str,
    domain: String,
) -> Result<Option<ClearingAccountConfig>, LocalError> {
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
    let user_clearing_acc: Option<ClearingAccountConfig> =
        serde_json::from_value(clearing_acc_response)
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?;

    info!(
        "user {user_addr} clearing account on {domain}: {:?}",
        user_clearing_acc
    );

    Ok(user_clearing_acc)
}

pub fn _query_user_config(
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

pub fn query_balance_query_id(
    test_ctx: &TestContext,
    orbital_core: String,
    query_id: u64,
) -> Result<neutron_sdk::interchain_queries::v047::queries::BalanceResponse, LocalError> {
    let query_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        &serde_json::to_string(&QueryMsg::Balance { query_id })
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
    )["data"]
        .clone();

    let balance_response: neutron_sdk::interchain_queries::v047::queries::BalanceResponse =
        serde_json::from_value(query_response).unwrap();

    info!("balance query response: {:?}", balance_response);

    Ok(balance_response)
}

pub fn query_icq_transfer_amount(
    test_ctx: &TestContext,
    orbital_core: String,
) -> Result<GetTransfersAmountResponse, LocalError> {
    let query_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        &serde_json::to_string(&QueryMsg::IcqTransfersAmount {})
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
    )["data"]
        .clone();

    let transfers_amount_response: GetTransfersAmountResponse =
        serde_json::from_value(query_response).unwrap();

    info!("ICQ transfer amount: {:?}", transfers_amount_response);

    Ok(transfers_amount_response)
}

pub fn query_icq_recipient_txs(
    test_ctx: &TestContext,
    orbital_core: String,
    recipient: String,
) -> Result<RecipientTxsResponse, LocalError> {
    let query_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        &serde_json::to_string(&QueryMsg::IcqRecipientTxs {
            recipient: recipient.to_string(),
        })
        .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
    )["data"]
        .clone();

    let recipient_txs_response: RecipientTxsResponse = serde_json::from_value(query_response)
        .map_err(|e| LocalError::Custom { msg: e.to_string() })?;

    info!(
        "ICQ recipient txs for {recipient}: {:?}",
        recipient_txs_response
    );

    Ok(recipient_txs_response)
}
