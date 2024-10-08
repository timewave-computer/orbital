use cosmwasm_std::{coin, coins};
use localic_std::modules::{bank::get_balance, cosmwasm::contract_instantiate};
use localic_utils::{
    ConfigChainBuilder, TestContextBuilder, GAIA_CHAIN_NAME, JUNO_CHAIN_NAME, NEUTRON_CHAIN_NAME,
};
use log::info;
use orbital_core::msg::InstantiateMsg;
use std::{env, error::Error, time::Duration};
use utils::{
    exec::{
        admin_register_domain, register_icq_balances_query, register_icq_transfers_query,
        user_register_orbital_core, user_register_to_new_domain, user_withdraw_funds_from_domain,
    },
    misc::{generate_icq_relayer_config, start_icq_relayer},
    query::{
        query_balance_query_id, query_icq_recipient_txs, query_icq_transfer_amount,
        query_user_clearing_acc_addr_on_domain,
    },
};

pub const POLYTONE_PATH: &str = "local-interchaintest/wasms/polytone";
pub const LOGS_FILE_PATH: &str = "local-interchaintest/configs/logs.json";
pub const LOCAL_CODE_ID_CACHE_PATH_NEUTRON: &str =
    "local-interchaintest/code_id_cache_neutron.json";

pub const ACC0_KEY: &str = "acc0";
pub const ACC0_ADDR: &str = "neutron1hj5fveer5cjtn4wd6wstzugjfdxzl0xpznmsky";
pub const ACC1_KEY: &str = "acc1";
pub const ACC1_ADDR: &str = "neutron1kljf09rj77uxeu5lye7muejx6ajsu55cuw2mws";
pub const ACC2_KEY: &str = "acc2";
pub const ACC2_ADDR: &str = "neutron17lp3n649rxt2jadn455frcj0q6anjnds2s0ve9";

pub const GAS_FLAGS: &str = "--gas=auto --gas-adjustment=3.0";
mod utils;

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

    let current_dir = env::current_dir()?;

    // with test context set up, we can generate the .env file for the icq relayer
    generate_icq_relayer_config(&test_ctx, current_dir.clone(), JUNO_CHAIN_NAME.to_string())?;

    // start the icq relayer. this runs in detached mode so we need
    // to manually kill it before each run for now.
    start_icq_relayer()?;

    let mut uploader = test_ctx.build_tx_upload_contracts();
    let orbital_core_local_path = format!("{}/artifacts/orbital_core.wasm", current_dir.display());

    info!("sleeping to allow icq relayer to start...");
    std::thread::sleep(Duration::from_secs(10));

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

    admin_register_domain(
        &test_ctx,
        orbital_core.address.to_string(),
        GAIA_CHAIN_NAME.to_string(),
    )?;
    std::thread::sleep(Duration::from_secs(2));
    admin_register_domain(
        &test_ctx,
        orbital_core.address.to_string(),
        JUNO_CHAIN_NAME.to_string(),
    )?;

    // first we register users to orbital-core
    user_register_orbital_core(&test_ctx, ACC1_KEY, orbital_core.address.to_string())?;
    user_register_orbital_core(&test_ctx, ACC2_KEY, orbital_core.address.to_string())?;

    std::thread::sleep(Duration::from_secs(3));

    // then we register them to gaia domain
    user_register_to_new_domain(
        &test_ctx,
        ACC1_KEY,
        orbital_core.address.to_string(),
        GAIA_CHAIN_NAME.to_string(),
    )?;
    user_register_to_new_domain(
        &test_ctx,
        ACC2_KEY,
        orbital_core.address.to_string(),
        GAIA_CHAIN_NAME.to_string(),
    )?;

    std::thread::sleep(Duration::from_secs(5));

    user_register_to_new_domain(
        &test_ctx,
        ACC1_KEY,
        orbital_core.address.to_string(),
        JUNO_CHAIN_NAME.to_string(),
    )?;
    user_register_to_new_domain(
        &test_ctx,
        ACC2_KEY,
        orbital_core.address.to_string(),
        JUNO_CHAIN_NAME.to_string(),
    )?;

    std::thread::sleep(Duration::from_secs(5));

    // query_user_config(&test_ctx, orbital_core.address.to_string(), ACC1_ADDR)?;

    let _acc_1_gaia_addr = query_user_clearing_acc_addr_on_domain(
        &test_ctx,
        orbital_core.address.to_string(),
        ACC1_ADDR,
        GAIA_CHAIN_NAME.to_string(),
    )?
    .unwrap()
    .addr;

    let acc_1_juno_addr = query_user_clearing_acc_addr_on_domain(
        &test_ctx,
        orbital_core.address.to_string(),
        ACC1_ADDR,
        JUNO_CHAIN_NAME.to_string(),
    )?
    .unwrap()
    .addr;

    std::thread::sleep(Duration::from_secs(5));

    let _acc_2_gaia_addr = query_user_clearing_acc_addr_on_domain(
        &test_ctx,
        orbital_core.address.to_string(),
        ACC2_ADDR,
        GAIA_CHAIN_NAME.to_string(),
    )?
    .unwrap()
    .addr;
    let acc_2_juno_addr = query_user_clearing_acc_addr_on_domain(
        &test_ctx,
        orbital_core.address.to_string(),
        ACC2_ADDR,
        JUNO_CHAIN_NAME.to_string(),
    )?
    .unwrap()
    .addr;

    std::thread::sleep(Duration::from_secs(5));

    let icq_registration_response = register_icq_balances_query(
        &test_ctx,
        orbital_core.address.to_string(),
        JUNO_CHAIN_NAME.to_string(),
        acc_1_juno_addr.to_string(),
        vec!["ujuno".to_string()],
    )?;

    info!("icq registration response: {:?}", icq_registration_response);

    std::thread::sleep(Duration::from_secs(5));

    let pre_transfer_balance = get_balance(
        test_ctx
            .get_request_builder()
            .get_request_builder(JUNO_CHAIN_NAME),
        acc_1_juno_addr.as_str(),
    );
    info!(
        "funding juno address. pre_transfer_balance: {:?}",
        pre_transfer_balance
    );

    let transfer_coins_str = coins(1_000_000, "ujuno")
        .iter()
        .map(|coin| format!("{}{}", coin.amount, coin.denom))
        .collect::<Vec<String>>()
        .join(",");

    let fee_coin = coin(50_000, "ujuno");

    test_ctx
        .get_request_builder()
        .get_request_builder(JUNO_CHAIN_NAME)
        .tx(&format!(
            "tx bank send {ACC0_KEY} {acc_1_juno_addr} {transfer_coins_str} --fees={fee_coin} --output=json"
        ), true)?;

    info!("sleeping for 5...");
    std::thread::sleep(Duration::from_secs(5));

    let balance_query_response =
        query_balance_query_id(&test_ctx, orbital_core.address.to_string(), 1)?;
    let post_transfer_balance = get_balance(
        test_ctx
            .get_request_builder()
            .get_request_builder(JUNO_CHAIN_NAME),
        acc_1_juno_addr.as_str(),
    );
    info!("ICQ balance query response  : {:?}", balance_query_response);
    info!("native bal query response   : {:?}", post_transfer_balance);

    info!("sleeping for 5...");
    std::thread::sleep(Duration::from_secs(5));

    info!("transfering more juno");
    test_ctx
        .get_request_builder()
        .get_request_builder(JUNO_CHAIN_NAME)
        .tx(&format!(
            "tx bank send {ACC0_KEY} {acc_1_juno_addr} {transfer_coins_str} --fees={fee_coin} --output=json"
        ), true)?;

    info!("sleeping for 5...");
    std::thread::sleep(Duration::from_secs(5));

    let user_2_juno_bal = get_balance(
        test_ctx
            .get_request_builder()
            .get_request_builder(JUNO_CHAIN_NAME),
        acc_2_juno_addr.as_str(),
    );
    info!("user 2 juno acc balance   : {:?}", user_2_juno_bal);

    user_withdraw_funds_from_domain(
        &test_ctx,
        orbital_core.address.to_string(),
        ACC1_KEY,
        JUNO_CHAIN_NAME.to_string(),
        acc_2_juno_addr.to_string(),
        1_000_000,
        "ujuno",
    )?;

    info!("sleeping for 5...");
    std::thread::sleep(Duration::from_secs(5));

    let balance_query_response =
        query_balance_query_id(&test_ctx, orbital_core.address.to_string(), 1)?;
    let post_transfer_balance = get_balance(
        test_ctx
            .get_request_builder()
            .get_request_builder(JUNO_CHAIN_NAME),
        acc_1_juno_addr.as_str(),
    );
    let user_2_juno_bal = get_balance(
        test_ctx
            .get_request_builder()
            .get_request_builder(JUNO_CHAIN_NAME),
        acc_2_juno_addr.as_str(),
    );
    info!(
        "user1 ICQ balance query response  : {:?}",
        balance_query_response
    );
    info!(
        "user1 native bal query response   : {:?}",
        post_transfer_balance
    );

    info!("user 2 juno acc balance   : {:?}", user_2_juno_bal);

    info!("sleeping for 5...");
    std::thread::sleep(Duration::from_secs(5));

    info!("registering ICQ transfer queries on juno");
    register_icq_transfers_query(
        &test_ctx,
        orbital_core.address.to_string(),
        JUNO_CHAIN_NAME.to_string(),
        acc_2_juno_addr.to_string(),
    )?;
    std::thread::sleep(Duration::from_secs(5));
    register_icq_transfers_query(
        &test_ctx,
        orbital_core.address.to_string(),
        JUNO_CHAIN_NAME.to_string(),
        acc_1_juno_addr.to_string(),
    )?;
    std::thread::sleep(Duration::from_secs(15));

    info!("user_2 withdrawing juno to user_1");

    user_withdraw_funds_from_domain(
        &test_ctx,
        orbital_core.address.to_string(),
        ACC2_KEY,
        JUNO_CHAIN_NAME.to_string(),
        acc_1_juno_addr.to_string(),
        1_000_000,
        "ujuno",
    )?;
    std::thread::sleep(Duration::from_secs(15));

    query_icq_recipient_txs(
        &test_ctx,
        orbital_core.address.to_string(),
        acc_1_juno_addr.to_string(),
    )?;
    std::thread::sleep(Duration::from_secs(5));

    query_icq_recipient_txs(
        &test_ctx,
        orbital_core.address.to_string(),
        acc_2_juno_addr.to_string(),
    )?;

    query_icq_transfer_amount(&test_ctx, orbital_core.address.to_string())?;

    Ok(())
}
