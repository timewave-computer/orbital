use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

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

pub fn query_registered_users(
    test_ctx: &TestContext,
    orbital_core: String,
) -> Result<(), LocalError> {
    let registered_users_query_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core,
        &serde_json::to_string(&QueryMsg::UserAddresses {})
            .map_err(|e| LocalError::Custom { msg: e.to_string() })?,
    )["data"]
        .clone();

    info!(
        "registered users query response: {:?}",
        registered_users_query_response
    );
    Ok(())
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
        "--amount 1000000untrn --gas 5000000",
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
        update_period: 1,
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

pub struct RelayerDetails {
    pub neutron_rpc: String,
    pub neutron_rest: String,
    pub home_dir: String,
    pub sign_key: String,
    pub connection_id: String,
    pub target_rpc: String,
}

pub fn generate_icq_relayer_config(
    current_path: PathBuf,
    relayer_details: RelayerDetails,
) -> std::io::Result<()> {
    // formatted according to neutron ICQ relayer docs
    let env_content = format!(
        r#"
RELAYER_NEUTRON_CHAIN_RPC_ADDR={neutron_rpc}
RELAYER_NEUTRON_CHAIN_REST_ADDR={neutron_rest}
RELAYER_NEUTRON_CHAIN_HOME_DIR={home_dir}
RELAYER_NEUTRON_CHAIN_SIGN_KEY_NAME={sign_key}
RELAYER_NEUTRON_CHAIN_TIMEOUT=10s
RELAYER_NEUTRON_CHAIN_GAS_PRICES=0.5untrn
RELAYER_NEUTRON_CHAIN_GAS_LIMIT=10000000
RELAYER_NEUTRON_CHAIN_GAS_ADJUSTMENT=2.0
RELAYER_NEUTRON_CHAIN_DENOM=untrn
RELAYER_NEUTRON_CHAIN_MAX_GAS_PRICE=1000
RELAYER_NEUTRON_CHAIN_GAS_PRICE_MULTIPLIER=1.1
RELAYER_NEUTRON_CHAIN_CONNECTION_ID={connection_id}
RELAYER_NEUTRON_CHAIN_DEBUG=true
RELAYER_NEUTRON_CHAIN_KEYRING_BACKEND=test
RELAYER_NEUTRON_CHAIN_OUTPUT_FORMAT=json
RELAYER_NEUTRON_CHAIN_SIGN_MODE_STR=direct

RELAYER_TARGET_CHAIN_RPC_ADDR={target_rpc}
RELAYER_TARGET_CHAIN_TIMEOUT=10s
RELAYER_TARGET_CHAIN_DEBUG=true
RELAYER_TARGET_CHAIN_OUTPUT_FORMAT=json

RELAYER_REGISTRY_ADDRESSES=
RELAYER_REGISTRY_QUERY_IDS=

RELAYER_ALLOW_TX_QUERIES=true
RELAYER_ALLOW_KV_CALLBACKS=true
RELAYER_MIN_KV_UPDATE_PERIOD=1
RELAYER_STORAGE_PATH=storage/leveldb
RELAYER_QUERIES_TASK_QUEUE_CAPACITY=10000
RELAYER_CHECK_SUBMITTED_TX_STATUS_DELAY=10s
RELAYER_INITIAL_TX_SEARCH_OFFSET=0
RELAYER_WEBSERVER_PORT=127.0.0.1:9999
RELAYER_IGNORE_ERRORS_REGEX=(execute wasm contract failed|failed to build tx query string)

#LOGGER_LEVEL=info
#LOGGER_OUTPUTPATHS=stdout, /tmp/logs
#LOGGER_ERROROUTPUTPATHS=stderr
"#,
        neutron_rpc = relayer_details.neutron_rpc,
        neutron_rest = relayer_details.neutron_rest,
        home_dir = relayer_details.home_dir,
        sign_key = relayer_details.sign_key,
        connection_id = relayer_details.connection_id,
        target_rpc = relayer_details.target_rpc
    );

    // create the env file and write the dynamically generated config there
    let path = current_path
        .join("local-interchaintest")
        .join("configs")
        .join(".env");
    let mut file = File::create(path)?;
    file.write_all(env_content.as_bytes())?;
    Ok(())
}
