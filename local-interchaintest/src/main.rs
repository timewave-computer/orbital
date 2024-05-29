#![allow(dead_code, unused_must_use)]

use std::path::Path;

use cosmwasm_std::{coin, to_json_string, Coin, Uint128};
use local_ictest_e2e::{
    pretty_print, utils::{
        file_system::{
            get_contract_cache_path, get_contract_path, get_current_dir, get_local_interchain_dir,
            read_json_file,
        },
        ibc::ibc_send,
        test_context::TestContext,
    }, ACC_0_KEY, API_URL, CHAIN_CONFIG_PATH, GAIA_CHAIN, JUNO_CHAIN, NEUTRON_CHAIN
};
use localic_std::{
    filesystem::get_files,
    modules::{bank::{get_balance, get_total_supply, send}, cosmwasm::CosmWasm},
    polling::poll_for_start,
    relayer::Relayer,
    transactions::ChainRequestBuilder, types::Contract,
};
use reqwest::blocking::Client;

use account::msg::QueryMsg as AccountQuery;
use account::msg::ExecuteMsg as AccountExecute;

// local-ic start neutron_gaia_juno
fn main() {


    let configured_chains = read_json_file(CHAIN_CONFIG_PATH).unwrap();

    let client = Client::new();
    poll_for_start(&client, API_URL, 300);

    let mut test_ctx = TestContext::from(configured_chains);

    // store polytunesss
    let contracts_path = get_contract_path();
    println!("main contracts path: {:?}", contracts_path);

    let artifacts_path = get_local_interchain_dir().join("artifacts");
    println!("main artifacts path: {:?}", artifacts_path);

    let polytone_path = contracts_path.join("polytone");
    let note_path = polytone_path.join("polytone_note.wasm");
    let voice_path = polytone_path.join("polytone_voice.wasm");
    let proxy_path = polytone_path.join("polytone_proxy.wasm");

    let orbital_contracts_path = contracts_path.join("orbital");
    let auction_path = orbital_contracts_path.join("auction-aarch64.wasm");
    let account_path = orbital_contracts_path.join("account-aarch64.wasm");

    let mut note_cw = test_ctx.get_cosmwasm_instance(NEUTRON_CHAIN);
    let mut account_cw = test_ctx.get_cosmwasm_instance(NEUTRON_CHAIN);
    let mut auction_cw = test_ctx.get_cosmwasm_instance(NEUTRON_CHAIN);

    let mut voice_cw = test_ctx.get_cosmwasm_instance(JUNO_CHAIN);
    let mut proxy_cw = test_ctx.get_cosmwasm_instance(JUNO_CHAIN);

    let key = "acc0";
    let neutron_rb = test_ctx.get_request_builder().get_request_builder(NEUTRON_CHAIN);
    let juno_rb = test_ctx.get_request_builder().get_request_builder(JUNO_CHAIN);
    
    let neutron_relayer = Relayer::new(&neutron_rb);
    let juno_relayer = Relayer::new(&juno_rb);

    let note_code_id = note_cw.store(key, &note_path).unwrap();
    let account_code_id = account_cw.store(key, &account_path).unwrap();
    let auction_code_id = auction_cw.store(key, &auction_path).unwrap();

    let voice_code_id = voice_cw.store(key, &voice_path).unwrap();
    let proxy_code_id = proxy_cw.store(key, &proxy_path).unwrap();

    println!("[NEUTRON] note code id: {:?}", note_code_id);
    println!("[NEUTRON] auction code id: {:?}", auction_code_id);
    println!("[NEUTRON] account code id: {:?}", account_code_id);

    println!("[JUNO]\t\tvoice code id: {:?}", voice_code_id);
    println!("[JUNO]\t\tproxy code id: {:?}", proxy_code_id);

    std::thread::sleep(std::time::Duration::from_secs(5));

    let note_contract = note_cw.instantiate(
        "acc0",
        "{\"block_max_gas\":\"3010000\"}",
        "neutron_note",
        None,
        ""
    ).unwrap();
    
    println!("note contract: {:?}", note_contract);

    let voice_contract = voice_cw.instantiate(
        "acc0",
        format!("{{\"proxy_code_id\":\"{}\",\"block_max_gas\":\"{}\"}}", proxy_code_id, "3010000").as_str(),
        "juno_voice",
        None,
        "",
    ).unwrap();
    println!("voice contract: {:?}", voice_contract);
    
    let polytone_channel_init = juno_relayer.create_channel(
        "neutron-juno",
        format!("wasm.{}", &note_contract.address).as_str(),
        format!("wasm.{}", &voice_contract.address).as_str(),
        "unordered",
        "polytone-1",
    ).unwrap();

    pretty_print(&polytone_channel_init);


    let account_contract = account_cw.instantiate(
        "acc0",
        "{}",
        "orbital_account",
        None,
        ""
    ).unwrap();

    println!("account contract: {:?}", account_contract);


    let msg = AccountExecute::RegisterDomain {
        domain: orbital_utils::domain::OrbitalDomain::Juno,
        note_addr: note_contract.address,
    };

    let register_domain_msg_str = to_json_string(&msg).unwrap();

    let resp = account_cw.execute(
        "acc0",
        &register_domain_msg_str,
        "--gas 5502650"
    ).unwrap();
    println!("resp: {:?}", resp);

    let query_domain_addr_msg = AccountQuery::QueryDomainAddr { domain: "juno".to_string() };
    let query_domain_addr_msg_str = to_json_string(&query_domain_addr_msg).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(20));

    let proxy_acc_query_msg_str = to_json_string(
        &AccountQuery::QueryProxyAccount { domain: "juno".to_string() }
    ).unwrap();
    let resp = account_cw.query(&proxy_acc_query_msg_str);
    let juno_proxy_address = resp["data"].as_str().unwrap();
    println!("juno proxy account address: {}", juno_proxy_address);
}

fn test_ibc_transfer(test_ctx: &TestContext) {
    let gaia = test_ctx.get_chain(GAIA_CHAIN);
    let neutron = test_ctx.get_chain(NEUTRON_CHAIN);
    let juno = test_ctx.get_chain(JUNO_CHAIN);

    let neutron_relayer = Relayer::new(&neutron.rb);
    let gaia_relayer = Relayer::new(&gaia.rb);
    let juno_relayer = Relayer::new(&juno.rb);

    let neutron_channels = neutron_relayer
        .get_channels(neutron.rb.chain_id.as_str())
        .unwrap();
    let gaia_channels = gaia_relayer
        .get_channels(gaia.rb.chain_id.as_str())
        .unwrap();
    let juno_channels = juno_relayer
        .get_channels(juno.rb.chain_id.as_str())
        .unwrap();

    println!("Neutron channels: {:?}", neutron_channels);
    println!("Gaia channels: {:?}", gaia_channels);
    println!("juno channels: {:?}", juno_channels);
}

fn test_bank_send(rb: &ChainRequestBuilder, src_addr: &str, denom: &str) {
    let before_bal = get_balance(rb, src_addr);

    let res = send(
        rb,
        ACC_0_KEY,
        src_addr,
        &[Coin {
            denom: denom.to_string(),
            amount: Uint128::new(5),
        }],
        &Coin {
            denom: denom.to_string(),
            amount: Uint128::new(5000),
        },
    );
    match res {
        Ok(res) => {
            println!("res: {res}");
        }
        Err(err) => {
            println!("err: {err}");
        }
    }

    let after_amount = get_balance(rb, src_addr);

    println!("before: {before_bal:?}");
    println!("after: {after_amount:?}");
}

fn test_queries(rb: &ChainRequestBuilder) {
    test_all_accounts(rb);
    let c = get_total_supply(rb);
    println!("total supply: {c:?}");
}

fn test_all_accounts(rb: &ChainRequestBuilder) {
    let res = rb.query("q auth accounts", false);
    println!("res: {res}");

    let Some(accounts) = res["accounts"].as_array() else {
        println!("No accounts found.");
        return;
    };

    for account in accounts.iter() {
        let acc_type = account["@type"].as_str().unwrap_or_default();

        let addr: &str = match acc_type {
            // "/cosmos.auth.v1beta1.ModuleAccount" => account["base_account"]["address"]
            "/cosmos.auth.v1beta1.ModuleAccount" => account.get("base_account").unwrap()["address"]
                .as_str()
                .unwrap_or_default(),
            _ => account["address"].as_str().unwrap_or_default(),
        };

        println!("{acc_type}: {addr}");
    }
}

pub fn test_paths(rb: &ChainRequestBuilder) {
    println!("current_dir: {:?}", get_current_dir());
    println!("local_interchain_dir: {:?}", get_local_interchain_dir());
    println!("contract_path: {:?}", get_contract_path());
    println!("contract_json_path: {:?}", get_contract_cache_path());

    // upload Makefile to the chain's home dir
    let arb_file = get_current_dir().join("Makefile");
    match rb.upload_file(&arb_file, true) {
        Ok(req_builder) => {
            let res = match req_builder.send() {
                Ok(r) => r,
                Err(err) => {
                    panic!("upload_file failed on request send {err:?}");
                }
            };
            let body = match res.text() {
                Ok(body) => body,
                Err(err) => {
                    panic!("upload_file failed on response body {err:?}");
                }
            };
            println!("body: {body:?}");
            let chain_id = rb.chain_id.to_string();
            let assertion_str = format!(
                "{{\"success\":\"file uploaded to {}\",\"location\":\"/var/cosmos-chain/{}/Makefile\"}}",
                chain_id, chain_id
            );
            assert_eq!(body, assertion_str);
        }
        Err(err) => {
            panic!("upload_file failed {err:?}");
        }
    };

    let files = match get_files(rb, format!("/var/cosmos-chain/{}", rb.chain_id).as_str()) {
        Ok(files) => files,
        Err(err) => {
            panic!("get_files failed {err:?}");
        }
    };

    assert!(files.contains(&"Makefile".to_string()));
    assert!(files.contains(&"config".to_string()));
    assert!(files.contains(&"data".to_string()));
    assert!(files.contains(&"keyring-test".to_string()));
    println!("files: {files:?}");
}
