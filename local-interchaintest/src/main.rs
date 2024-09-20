use cosmwasm_std::Uint64;
use localic_std::modules::cosmwasm::{contract_execute, contract_instantiate, contract_query};
use localic_utils::{
    ConfigChainBuilder, TestContextBuilder, GAIA_CHAIN_NAME, JUNO_CHAIN_NAME, LOCAL_IC_API_URL,
    NEUTRON_CHAIN_NAME,
};
use log::info;
use orbital_core::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    orbital_domain::UncheckedOrbitalDomainConfig,
};
use std::{env, error::Error, time::Duration};

pub const POLYTONE_PATH: &str = "local-interchaintest/wasms/polytone";
pub const LOGS_FILE_PATH: &str = "local-interchaintest/configs/logs.json";
pub const LOCAL_CODE_ID_CACHE_PATH_NEUTRON: &str =
    "local-interchaintest/code_id_cache_neutron.json";

pub const ACC0_KEY: &str = "acc0";
pub const ACC0_ADDR: &str = "neutron1hj5fveer5cjtn4wd6wstzugjfdxzl0xpznmsky";
pub const ACC1_KEY: &str = "acc1";
pub const ACC1_ADDR: &str = "neutron1kljf09rj77uxeu5lye7muejx6ajsu55cuw2mws";
pub const GAS_FLAGS: &str = "--gas=auto --gas-adjustment=3.0";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut test_ctx = TestContextBuilder::default()
        .with_unwrap_raw_logs(true)
        .with_api_url("http://localhost:42069/")
        .with_artifacts_dir("artifacts")
        .with_chain(ConfigChainBuilder::default_neutron().build()?)
        .with_chain(ConfigChainBuilder::default_gaia().build()?)
        .with_chain(ConfigChainBuilder::default_juno().build()?)
        .with_log_file_path(LOGS_FILE_PATH)
        .with_transfer_channels(NEUTRON_CHAIN_NAME, GAIA_CHAIN_NAME)
        .with_transfer_channels(NEUTRON_CHAIN_NAME, JUNO_CHAIN_NAME)
        .build()?;

    let mut uploader = test_ctx.build_tx_upload_contracts();

    // TODO: uncomment to deploy polytone
    // uploader
    //     .with_key(ACC0_KEY)
    //     .send_with_local_cache(POLYTONE_PATH, LOCAL_CODE_ID_CACHE_PATH_NEUTRON)
    //     .unwrap();

    let current_dir = env::current_dir()?;

    let orbital_core_local_path = format!("{}/artifacts/orbital_core.wasm", current_dir.display());

    uploader
        // .with_key(ACC0_KEY)
        .with_chain_name(NEUTRON_CHAIN_NAME)
        .send_single_contract(&orbital_core_local_path)?;

    let orbital_core_code_id = test_ctx
        .get_contract()
        .contract("orbital_core")
        .get_cw()
        .code_id
        .unwrap();

    info!("orbital core code id: {orbital_core_code_id}");

    let orbital_instantiate_msg = InstantiateMsg {
        owner: test_ctx
            .get_chain(NEUTRON_CHAIN_NAME)
            .admin_addr
            .to_string(),
    };

    // instantiate orbital-core from the ACC0_KEY (=admin in localic-utils)
    let orbital_core = contract_instantiate(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        ACC0_KEY,
        orbital_core_code_id,
        &serde_json::to_string(&orbital_instantiate_msg)?,
        "orbital_core",
        None,
        "",
    )?;

    info!("orbital core: {}", orbital_core.address);

    let admin_register_gaia_domain_msg = ExecuteMsg::RegisterNewDomain {
        domain: GAIA_CHAIN_NAME.to_string(),
        account_type: UncheckedOrbitalDomainConfig::InterchainAccount {
            connection_id: test_ctx
                .get_connections()
                .src(NEUTRON_CHAIN_NAME)
                .dest(GAIA_CHAIN_NAME)
                .get(),
            channel_id: test_ctx
                .get_transfer_channels()
                .src(NEUTRON_CHAIN_NAME)
                .dest(GAIA_CHAIN_NAME)
                .get(),
            timeout: Uint64::new(100),
        },
    };

    // register the gaia domain on orbital-core (admin gated)
    info!("admin registering orbital-level gaia domain...");
    contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        ACC0_KEY,
        &serde_json::to_string(&admin_register_gaia_domain_msg)?,
        "",
    )?;

    std::thread::sleep(Duration::from_secs(5));

    // user 1 registers to orbital-core
    info!("user_1 registering to orbital-core...");
    contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        ACC1_KEY,
        &serde_json::to_string(&ExecuteMsg::RegisterUser {})?,
        "",
    )?;

    std::thread::sleep(Duration::from_secs(5));

    // user 1 registers to gaia domain
    info!("user_1 registering to gaia domain...");
    let user_domain_registration_resp = contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        ACC1_KEY,
        &serde_json::to_string(&ExecuteMsg::RegisterUserDomain {
            domain: GAIA_CHAIN_NAME.to_string(),
        })?,
        "--amount 1000000untrn --gas 5000000",
    )?;
    std::thread::sleep(Duration::from_secs(5));

    info!(
        "user domain registration response: {:?}",
        user_domain_registration_resp
    );

    let tx_res = test_ctx
        .get_request_builder()
        .get_request_builder(NEUTRON_CHAIN_NAME)
        .query_tx_hash(&user_domain_registration_resp.tx_hash.unwrap());

    info!("tx hash response: {:?}", tx_res);

    let registered_users_query_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        &serde_json::to_string(&QueryMsg::UserAddresses {})?,
    )["data"]
        .clone();

    info!(
        "registered users query response: {:?}",
        registered_users_query_response
    );

    let query_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        &serde_json::to_string(&QueryMsg::UserConfig {
            addr: ACC1_ADDR.to_string(),
        })?,
    )["data"]
        .clone();
    info!("user config query response: {:?}", query_response);

    let clearing_acc_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        &serde_json::to_string(&QueryMsg::ClearingAccountAddress {
            addr: ACC1_ADDR.to_string(),
            domain: GAIA_CHAIN_NAME.to_string(),
        })?,
    )["data"]
        .clone();
    info!(
        "clearing account query response: {:?}",
        clearing_acc_response
    );
    let user_gaia_clearing_acc: Option<String> = serde_json::from_value(clearing_acc_response)?;

    info!("user gaia clearing account: {:?}", user_gaia_clearing_acc);

    Ok(())
}
