use cosmwasm_std::Uint64;
use localic_std::modules::cosmwasm::{contract_execute, contract_instantiate};
use localic_utils::{
    ConfigChainBuilder, TestContextBuilder, DEFAULT_KEY, GAIA_CHAIN_NAME, JUNO_CHAIN_NAME,
    LOCAL_IC_API_URL, NEUTRON_CHAIN_NAME,
};
use log::info;
use orbital_core::account_types::AccountConfigType;
use std::{env, error::Error};

pub const POLYTONE_PATH: &str = "local-interchaintest/wasms/polytone";
pub const LOGS_FILE_PATH: &str = "local-interchaintest/configs/logs.json";
pub const LOCAL_CODE_ID_CACHE_PATH_NEUTRON: &str =
    "local-interchaintest/code_id_cache_neutron.json";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut test_ctx = TestContextBuilder::default()
        .with_unwrap_raw_logs(true)
        .with_api_url(LOCAL_IC_API_URL)
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
        account_type: AccountConfigType::ICA {
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
    let resp = contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        DEFAULT_KEY,
        &serde_json::to_string(&register_gaia_domain_msg).unwrap(),
        "",
    )
    .unwrap();

    let user_registration_resp = contract_execute(
        test_ctx
            .get_request_builder()
            .get_request_builder(NEUTRON_CHAIN_NAME),
        &orbital_core.address,
        "acc1",
        &serde_json::to_string(&orbital_core::msg::ExecuteMsg::RegisterUser {}).unwrap(),
        "",
    )
    .unwrap();

    Ok(())
}
