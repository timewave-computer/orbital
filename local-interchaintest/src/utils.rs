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

pub fn generate_icq_relayer_config(
    test_ctx: &TestContext,
    current_path: PathBuf,
    target_domain: String,
) -> std::io::Result<()> {
    let target_connection_id = test_ctx
        .get_connections()
        .src(NEUTRON_CHAIN_NAME)
        .dest(&target_domain)
        .get();

    // formatted according to neutron ICQ relayer docs
    let target_chain_rpc = format!(
        "tcp://local{}-1-val-0-neutron_gaia_junoic:26657",
        target_domain
    );
    let env_content = format!(
        r#"
RELAYER_NEUTRON_CHAIN_RPC_ADDR={neutron_rpc}
RELAYER_NEUTRON_CHAIN_REST_ADDR={neutron_rest}
RELAYER_NEUTRON_CHAIN_HOME_DIR=/data
RELAYER_NEUTRON_CHAIN_SIGN_KEY_NAME=acc3
RELAYER_NEUTRON_CHAIN_GAS_PRICES=0.5untrn
RELAYER_NEUTRON_CHAIN_GAS_LIMIT=10000000
RELAYER_NEUTRON_CHAIN_GAS_ADJUSTMENT=2.0
RELAYER_NEUTRON_CHAIN_DENOM=untrn
RELAYER_NEUTRON_CHAIN_MAX_GAS_PRICE=1000
RELAYER_NEUTRON_CHAIN_GAS_PRICE_MULTIPLIER=1.1
RELAYER_NEUTRON_CHAIN_CONNECTION_ID={connection_id}
RELAYER_NEUTRON_CHAIN_DEBUG=true
RELAYER_NEUTRON_CHAIN_KEYRING_BACKEND=test
RELAYER_NEUTRON_CHAIN_ACCOUNT_PREFIX=neutron
RELAYER_NEUTRON_CHAIN_KEY=acc3
RELAYER_NEUTRON_CHAIN_OUTPUT_FORMAT=json
RELAYER_NEUTRON_CHAIN_SIGN_MODE_STR=direct

RELAYER_TARGET_CHAIN_RPC_ADDR={target_rpc}
RELAYER_TARGET_CHAIN_TIMEOUT=10s
RELAYER_TARGET_CHAIN_DEBUG=true
RELAYER_TARGET_CHAIN_KEYRING_BACKEND=test
RELAYER_TARGET_CHAIN_OUTPUT_FORMAT=json

RELAYER_REGISTRY_ADDRESSES=
RELAYER_REGISTRY_QUERY_IDS=

RELAYER_ALLOW_TX_QUERIES=true
RELAYER_ALLOW_KV_CALLBACKS=true
RELAYER_STORAGE_PATH=storage/leveldb
RELAYER_WEBSERVER_PORT=127.0.0.1:9999
RELAYER_IGNORE_ERRORS_REGEX=(execute wasm contract failed|failed to build tx query string)
"#,
        neutron_rpc = "tcp://localneutron-1-val-0-neutron_gaia_junoic:26657",
        neutron_rest = "http://localneutron-1-val-0-neutron_gaia_junoic:1317",
        connection_id = target_connection_id,
        target_rpc = target_chain_rpc,
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

pub fn start_icq_relayer() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("docker")
        .arg("inspect")
        .arg("localneutron-1-val-0-neutron_gaia_junoic")
        .output()
        .expect("failed to inspect the neutron container");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse JSON from docker inspect output");

    // extract the docker network under which neutron container is spinning
    let docker_network = response[0]["NetworkSettings"]["Networks"].clone();
    let network_name = docker_network
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .to_string();

    // extract the mount point of neutron chain on host machine
    let mount_point = response[0]["Mounts"][0]["Source"].as_str().unwrap();

    // this should be initiated by `just local-ic-run`, so we know the relpath
    let env_relpath = "local-interchaintest/configs/.env";

    let start_icq_relayer_cmd = std::process::Command::new("docker")
        .arg("run")
        .arg("-d") // detached mode to not block the main()
        .arg("--env-file")
        .arg(env_relpath) // passing the .env file we generated before
        .arg("-p")
        .arg("9999:9999") // the port binding for the relayer webserver, idk if it's needed
        .arg("--network")
        .arg(network_name) // docker network under which we want to run the relayer
        .arg("-v")
        .arg(format!("{}:/data", mount_point)) // neutron mount point to access the keyring
        .arg("neutron-org/neutron-query-relayer")
        .output()
        .expect("failed to start icq relayer");

    match start_icq_relayer_cmd.status.success() {
        true => {
            let container_id = String::from_utf8_lossy(&start_icq_relayer_cmd.stdout)
                .trim()
                .to_string();
            info!("ICQ relayer container started with ID: {}", container_id);
            Ok(())
        }
        false => {
            let error = String::from_utf8_lossy(&start_icq_relayer_cmd.stderr);
            Err(format!("Failed to start ICQ relayer: {}", error).into())
        }
    }
}
