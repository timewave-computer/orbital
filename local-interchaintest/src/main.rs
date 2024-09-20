use cosmwasm_std::Uint64;
use localic_std::modules::cosmwasm::{contract_execute, contract_instantiate, contract_query};
use localic_utils::{
    ConfigChainBuilder, TestContextBuilder, DEFAULT_KEY, GAIA_CHAIN_NAME, JUNO_CHAIN_NAME,
    NEUTRON_CHAIN_NAME,
};
use log::info;
use orbital_core::{
    msg::QueryMsg, orbital_domain::UncheckedOrbitalDomainConfig, state::UserConfig,
};
use std::{env, error::Error, time::Duration};

pub const POLYTONE_PATH: &str = "local-interchaintest/wasms/polytone";
pub const LOGS_FILE_PATH: &str = "local-interchaintest/configs/logs.json";
pub const LOCAL_CODE_ID_CACHE_PATH_NEUTRON: &str =
    "local-interchaintest/code_id_cache_neutron.json";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut test_ctx = TestContextBuilder::default()
        .with_unwrap_raw_logs(true)
        .with_api_url("http://localhost:8080/")
        .with_artifacts_dir("artifacts")
        .with_chain(ConfigChainBuilder::default_neutron().build()?)
        .with_chain(ConfigChainBuilder::default_gaia().build()?)
        .with_chain(ConfigChainBuilder::default_juno().build()?)
        .with_log_file_path(LOGS_FILE_PATH)
        .with_transfer_channels(NEUTRON_CHAIN_NAME, GAIA_CHAIN_NAME)
        .with_transfer_channels(NEUTRON_CHAIN_NAME, JUNO_CHAIN_NAME)
        .build()?;

    let mut uploader = test_ctx.build_tx_upload_contracts();

    uploader
        .send_with_local_cache(POLYTONE_PATH, LOCAL_CODE_ID_CACHE_PATH_NEUTRON)
        .unwrap();

    let current_dir = env::current_dir()?;

    let orbital_core_local_path = format!("{}/artifacts/orbital_core.wasm", current_dir.display());

    uploader
        .with_chain_name(NEUTRON_CHAIN_NAME)
        .send_single_contract(&orbital_core_local_path)?;

    let orbital_core_code_id = test_ctx
        .get_contract()
        .contract("orbital_core")
        .get_cw()
        .code_id
        .unwrap();

    info!("orbital core code id: {orbital_core_code_id}");

    let orbital_instantiate_msg = orbital_core::msg::InstantiateMsg {
        owner: test_ctx
            .get_chain(NEUTRON_CHAIN_NAME)
            .admin_addr
            .to_string(),
    };

    let orbital_core = contract_instantiate(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        DEFAULT_KEY,
        orbital_core_code_id,
        &serde_json::to_string(&orbital_instantiate_msg).unwrap(),
        "orbital_core",
        None,
        "",
    )
    .unwrap();

    info!("orbital core: {}", orbital_core.address);

    let register_gaia_domain_msg = orbital_core::msg::ExecuteMsg::RegisterNewDomain {
        domain: "gaia".to_string(),
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
    let _resp = contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        DEFAULT_KEY,
        &serde_json::to_string(&register_gaia_domain_msg).unwrap(),
        "",
    )
    .unwrap();
    info!("registered gaia domain");
    std::thread::sleep(Duration::from_secs(5));
    let _user_registration_resp = contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        "acc1",
        &serde_json::to_string(&orbital_core::msg::ExecuteMsg::RegisterUser {}).unwrap(),
        "",
    )
    .unwrap();
    info!("registered user acc1");

    std::thread::sleep(Duration::from_secs(5));
    let user_domain_registration_resp = contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        "acc1",
        &serde_json::to_string(&orbital_core::msg::ExecuteMsg::RegisterUserDomain {
            domain: "gaia".to_string(),
        })
        .unwrap(),
        "--gas 100000000",
    )
    .unwrap();
    info!("registered user acc1 to gaia domain");
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
        &serde_json::to_string(&QueryMsg::UserAddresses {}).unwrap(),
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
            addr: "neutron1kljf09rj77uxeu5lye7muejx6ajsu55cuw2mws".to_string(),
        })
        .unwrap(),
    )["data"]
        .clone();
    info!("user config query response: {:?}", query_response);
    // let registered_user_config: UserConfig = serde_json::from_value(query_response).unwrap();

    // info!("registered user config: {:?}", registered_user_config);

    let clearing_acc_response = contract_query(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        &serde_json::to_string(&QueryMsg::ClearingAccountAddress {
            addr: "neutron1kljf09rj77uxeu5lye7muejx6ajsu55cuw2mws".to_string(),
            domain: "gaia".to_string(),
        })
        .unwrap(),
    )["data"]
        .clone();
    info!(
        "clearing account query response: {:?}",
        clearing_acc_response
    );
    let user_gaia_clearing_acc: Option<String> =
        serde_json::from_value(clearing_acc_response).unwrap();

    info!("user gaia clearing account: {:?}", user_gaia_clearing_acc);

    Ok(())
}
