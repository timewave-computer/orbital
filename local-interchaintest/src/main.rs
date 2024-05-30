#![allow(dead_code, unused_must_use)]

use cosmwasm_std::{coin, to_json_string, Coin, Uint128};
use cw_utils::Duration as CwDuration;
use local_ictest_e2e::{
    pretty_print,
    utils::{
        file_system::{get_contract_path, read_json_file},
        test_context::TestContext,
    },
    API_URL, CHAIN_CONFIG_PATH, JUNO_CHAIN, MM_KEY, NEUTRON_CHAIN,
};
use localic_std::{modules::cosmwasm::CosmWasm, polling::poll_for_start, relayer::Relayer};
use orbital_utils::domain::OrbitalDomain;
use reqwest::blocking::Client;

use account::msg::ExecuteMsg as AccountExecute;
use account::msg::QueryMsg as AccountQuery;

use auction::msg::ExecuteMsg as AuctionExecute;
use auction::msg::InstantiateMsg as AuctionInstantiate;

pub const MM_JUNO_ADDR: &str = "juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk";
pub const MM_NEUTRON_ADDR: &str = "neutron1efd63aw40lxf3n4mhf7dzhjkr453axur78g5ld";
pub const USER_KEY: &str = "acc0";

pub fn register_new_domain(cw: &CosmWasm, domain: OrbitalDomain, note_addr: String) {
    let msg = AccountExecute::RegisterDomain { domain, note_addr };

    let register_domain_msg_str = to_json_string(&msg).unwrap();
    let resp = cw
        .execute(USER_KEY, &register_domain_msg_str, "--gas 5502650")
        .unwrap();
    println!("register new domain tx: {:?}", resp.tx_hash);
}

// local-ic start neutron_gaia_juno
fn main() {
    let configured_chains = read_json_file(CHAIN_CONFIG_PATH).unwrap();

    let client = Client::new();
    poll_for_start(&client, API_URL, 300);

    let test_ctx = TestContext::from(configured_chains);

    // store polytunesss
    let contracts_path = get_contract_path();
    println!("main contracts path: {:?}", contracts_path);

    let polytone_path = contracts_path.join("polytone");
    let note_path = polytone_path.join("polytone_note.wasm");
    let voice_path = polytone_path.join("polytone_voice.wasm");
    let proxy_path = polytone_path.join("polytone_proxy.wasm");

    let orbital_contracts_path = contracts_path.join("orbital");
    let auction_path = orbital_contracts_path.join("auction.wasm");
    let account_path = orbital_contracts_path.join("account.wasm");

    let mut note_cw = test_ctx.get_cosmwasm_instance(NEUTRON_CHAIN);
    let mut account_cw = test_ctx.get_cosmwasm_instance(NEUTRON_CHAIN);
    let mut auction_cw = test_ctx.get_cosmwasm_instance(NEUTRON_CHAIN);

    let mut voice_cw = test_ctx.get_cosmwasm_instance(JUNO_CHAIN);
    let mut proxy_cw = test_ctx.get_cosmwasm_instance(JUNO_CHAIN);

    let neutron_rb = test_ctx
        .get_request_builder()
        .get_request_builder(NEUTRON_CHAIN);
    let juno_rb = test_ctx
        .get_request_builder()
        .get_request_builder(JUNO_CHAIN);

    let _neutron_relayer = Relayer::new(neutron_rb);
    let juno_relayer = Relayer::new(juno_rb);

    let note_code_id = note_cw.store(USER_KEY, &note_path).unwrap();
    let account_code_id = account_cw.store(USER_KEY, &account_path).unwrap();
    let auction_code_id = auction_cw.store(USER_KEY, &auction_path).unwrap();

    let voice_code_id = voice_cw.store(USER_KEY, &voice_path).unwrap();
    let proxy_code_id = proxy_cw.store(USER_KEY, &proxy_path).unwrap();

    println!("[NEUTRON] note code id: {:?}", note_code_id);
    println!("[NEUTRON] auction code id: {:?}", auction_code_id);
    println!("[NEUTRON] account code id: {:?}", account_code_id);

    println!("[JUNO]\t\tvoice code id: {:?}", voice_code_id);
    println!("[JUNO]\t\tproxy code id: {:?}", proxy_code_id);

    std::thread::sleep(std::time::Duration::from_secs(5));

    let note_contract = note_cw
        .instantiate(
            USER_KEY,
            "{\"block_max_gas\":\"3010000\"}",
            "neutron_note",
            None,
            "",
        )
        .unwrap();

    println!("note contract: {:?}", note_contract);

    let voice_contract = voice_cw
        .instantiate(
            USER_KEY,
            format!(
                "{{\"proxy_code_id\":\"{}\",\"block_max_gas\":\"{}\"}}",
                proxy_code_id, "3010000"
            )
            .as_str(),
            "juno_voice",
            None,
            "",
        )
        .unwrap();
    println!("voice contract: {:?}", voice_contract);

    let polytone_channel_init = juno_relayer
        .create_channel(
            "neutron-juno",
            format!("wasm.{}", &note_contract.address).as_str(),
            format!("wasm.{}", &voice_contract.address).as_str(),
            "unordered",
            "polytone-1",
        )
        .unwrap();

    pretty_print("polytone channel init response", &polytone_channel_init);

    let account_contract = account_cw
        .instantiate(USER_KEY, "{}", "orbital_account", None, "")
        .unwrap();

    println!("account contract: {:?}", account_contract);

    register_new_domain(
        &account_cw,
        OrbitalDomain::Juno,
        note_contract.address.to_string(),
    );
    // let resp = account_cw
    //     .execute(USER_KEY, &register_domain_msg, "--gas 5502650")
    //     .unwrap();
    // println!("resp: {:?}", resp);

    std::thread::sleep(std::time::Duration::from_secs(20));

    let proxy_acc_query_msg_str = to_json_string(&AccountQuery::QueryProxyAccount {
        domain: "juno".to_string(),
    })
    .unwrap();
    let resp = account_cw.query(&proxy_acc_query_msg_str);
    let juno_proxy_address = resp["data"].as_str().unwrap();
    println!("juno proxy account address: {}", juno_proxy_address);

    let proxy_acc_ledger_query_msg_str = to_json_string(&AccountQuery::QueryLedger {
        domain: "juno".to_string(),
    })
    .unwrap();

    let resp = account_cw.query(&proxy_acc_ledger_query_msg_str);
    println!("juno proxy account ledger response: {:?}", resp);

    let _fund_proxy = localic_std::modules::bank::send(
        juno_rb,
        USER_KEY,
        juno_proxy_address,
        &[coin(100_000, "ujuno")],
        &coin(1_000, "ujuno"),
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let sync_juno_domain_msg = AccountExecute::Sync {
        domain: OrbitalDomain::Juno,
    };
    let sync_juno_domain_msg_str = to_json_string(&sync_juno_domain_msg).unwrap();

    let resp = account_cw
        .execute(USER_KEY, &sync_juno_domain_msg_str, "--gas 5502650")
        .unwrap();
    println!("sync_juno_domain_msg_response: {:?}", resp);
    std::thread::sleep(std::time::Duration::from_secs(10));

    let proxy_acc_ledger_query_msg_str = to_json_string(&AccountQuery::QueryLedger {
        domain: "juno".to_string(),
    })
    .unwrap();

    let resp = account_cw.query(&proxy_acc_ledger_query_msg_str);
    pretty_print("ledger query response", &resp);

    let withdraw_msg = AccountExecute::WithdrawFunds {
        domain: OrbitalDomain::Juno,
        coin: Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::new(1),
        },
        dest: MM_JUNO_ADDR.to_string(),
    };

    let bal = localic_std::modules::bank::get_balance(juno_rb, MM_JUNO_ADDR);
    println!("juno mm balance: {:?}", bal);
    println!("\n withdrawing funds from juno domain to mm address\n");

    let withdraw_funds_resp = account_cw
        .execute(
            USER_KEY,
            to_json_string(&withdraw_msg).unwrap().as_str(),
            "--gas 5502650",
        )
        .unwrap();
    println!("withdraw_funds_resp: {:?}", withdraw_funds_resp);

    std::thread::sleep(std::time::Duration::from_secs(15));

    let bal = localic_std::modules::bank::get_balance(juno_rb, MM_JUNO_ADDR);
    println!("juno mm balance: {:?}", bal);

    let proxy_acc_ledger_query_msg_str = to_json_string(&AccountQuery::QueryLedger {
        domain: "juno".to_string(),
    })
    .unwrap();

    let resp = account_cw.query(&proxy_acc_ledger_query_msg_str);
    pretty_print("ledger query response", &resp);

    let auction_contract = auction_cw
        .instantiate(
            USER_KEY,
            to_json_string(&AuctionInstantiate {
                account_addr: account_contract.address.clone(),
                bond: coin(100, "untrn"),
                increment_bps: 10,
                duration: CwDuration::Time(30),
                fulfillment_timeout: CwDuration::Time(30),
            })
            .unwrap()
            .as_str(),
            "orbital_auction",
            None,
            "",
        )
        .unwrap();

    println!("auction contract: {:?}", auction_contract);
    std::thread::sleep(std::time::Duration::from_secs(5));

    let response = account_cw
        .execute(
            USER_KEY,
            &to_json_string(&AccountExecute::UpdateAuctionAddr {
                auction_addr: auction_contract.address.clone(),
            })
            .unwrap(),
            "",
        )
        .unwrap();
    println!("update auction response: {:?}", response);
    std::thread::sleep(std::time::Duration::from_secs(5));

    let new_intent_msg = AccountExecute::NewIntent(orbital_utils::intent::Intent {
        ask_domain: OrbitalDomain::Neutron,
        ask_coin: coin(10, "untrn"),
        offer_domain: OrbitalDomain::Juno,
        offer_coin: coin(100, "ujuno"),
        is_verified: false,
    });

    let response = account_cw
        .execute(USER_KEY, &to_json_string(&new_intent_msg).unwrap(), "")
        .unwrap();
    println!("create new intent response: {:?}", response);
    std::thread::sleep(std::time::Duration::from_secs(5));

    let new_tick_msg = AuctionExecute::AuctionTick {};
    let response = auction_cw
        .execute("acc0", &to_json_string(&new_tick_msg).unwrap(), "")
        .unwrap();
    println!("tick auction response: {:?}", response);
    std::thread::sleep(std::time::Duration::from_secs(5));

    let new_bond_msg = AuctionExecute::Bond {};
    let response = auction_cw
        .execute(
            MM_KEY,
            &to_json_string(&new_bond_msg).unwrap(),
            "--amount 100untrn",
        )
        .unwrap();
    println!("MM bond response: {:?}", response);
    std::thread::sleep(std::time::Duration::from_secs(5));

    let bid_msg = AuctionExecute::AuctionBid {
        bidder: MM_JUNO_ADDR.to_string(),
        bid: Uint128::new(100),
    };
    let response = auction_cw
        .execute(MM_KEY, &to_json_string(&bid_msg).unwrap(), "")
        .unwrap();
    println!("bid response: {:?}", response);
    std::thread::sleep(std::time::Duration::from_secs(25));

    let new_tick_msg = AuctionExecute::AuctionTick {};
    let response = auction_cw
        .execute("acc0", &to_json_string(&new_tick_msg).unwrap(), "")
        .unwrap();
    println!("tick auction response: {:?}", response);
    std::thread::sleep(std::time::Duration::from_secs(5));

    let res = localic_std::modules::bank::send(
        neutron_rb,
        MM_KEY,
        MM_NEUTRON_ADDR,
        &[coin(100, "untrn")],
        &coin(100, "untrn"),
    )
    .unwrap();
    pretty_print("bank send res", &res);
    std::thread::sleep(std::time::Duration::from_secs(5));

    let new_tick_msg = AuctionExecute::AuctionTick {};
    let response = auction_cw
        .execute("acc0", &to_json_string(&new_tick_msg).unwrap(), "")
        .unwrap();
    println!("tick auction response: {:?}", response);
    std::thread::sleep(std::time::Duration::from_secs(5));
}

// D - init an auction
// D - start new intent
// D - bid
// D - sleep until auction ends
// D - call auction tick
// mm deposit into given addr
// verify auction
