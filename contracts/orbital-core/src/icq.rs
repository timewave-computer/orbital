use cosmos_sdk_proto::{
    cosmos::{
        bank::v1beta1::MsgSend,
        tx::v1beta1::{TxBody, TxRaw},
    },
    prost::Message,
};
use cosmwasm_std::{Binary, DepsMut, Env, Response};
use cw_storage_plus::{Item, Map};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery, types::Height},
    interchain_queries::{get_registered_query, v045::new_register_balances_query_msg},
    NeutronResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{StdError, StdResult, Uint128};

use neutron_sdk::bindings::query::QueryRegisteredQueryResponse;
use neutron_sdk::interchain_queries::v047::types::{COSMOS_SDK_TRANSFER_MSG_URL, RECIPIENT_FIELD};

use neutron_sdk::interchain_queries::types::{
    TransactionFilterItem, TransactionFilterOp, TransactionFilterValue,
};
use serde_json_wasm;

/// defines the incoming transfers limit to make a case of failed callback possible.
const MAX_ALLOWED_TRANSFER: u64 = 20000;
const MAX_ALLOWED_MESSAGES: usize = 20;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cw20BalanceResponse {
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetRecipientTxsResponse {
    pub transfers: Vec<Transfer>,
}
pub type Recipient = str;
/// contains all transfers mapped by a recipient address observed by the contract.
pub const RECIPIENT_TXS: Map<&Recipient, Vec<Transfer>> = Map::new("recipient_txs");
/// contains number of transfers to addresses observed by the contract.
pub const TRANSFERS: Item<u64> = Item::new("transfers");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Transfer {
    pub recipient: String,
    pub sender: String,
    pub denom: String,
    pub amount: String,
}

pub fn register_balances_query(
    connection_id: String,
    addr: String,
    denoms: Vec<String>,
    update_period: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg = new_register_balances_query_msg(connection_id, addr, denoms, update_period)?;

    Ok(Response::new().add_message(msg))
}

/// sudo_check_tx_query_result is an example callback for transaction query results that stores the
/// deposits received as a result on the registered query in the contract's state.
pub fn sudo_tx_query_result(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    query_id: u64,
    _height: Height,
    data: Binary,
) -> StdResult<Response<NeutronMsg>> {
    // Decode the transaction data
    let tx: TxRaw = TxRaw::decode(data.as_slice())
        .map_err(|_| StdError::generic_err("sudo_tx_query_result failed to decode tx_raw"))?;
    let body: TxBody = TxBody::decode(tx.body_bytes.as_slice())
        .map_err(|_| StdError::generic_err("sudo_tx_query_result failed to decode tx_body"))?;

    // Get the registered query by ID and retrieve the raw query string
    let registered_query: QueryRegisteredQueryResponse =
        get_registered_query(deps.as_ref(), query_id).map_err(|_| {
            StdError::generic_err("sudo_tx_query_result failed to get registered query response")
        })?;
    let transactions_filter = registered_query.registered_query.transactions_filter;

    #[allow(clippy::match_single_binding)]
    // Depending of the query type, check the transaction data to see whether is satisfies
    // the original query. If you don't write specific checks for a transaction query type,
    // all submitted results will be treated as valid.
    //
    // TODO: come up with solution to determine transactions filter type
    match registered_query.registered_query.query_type {
        _ => {
            // For transfer queries, query data looks like `[{"field:"transfer.recipient", "op":"eq", "value":"some_address"}]`
            let query_data: Vec<TransactionFilterItem> =
                serde_json_wasm::from_str(transactions_filter.as_str()).map_err(|_| {
                    StdError::generic_err("sudo_tx_query_result failed to parse tx query type")
                })?;

            let recipient = query_data
                .iter()
                .find(|x| x.field == RECIPIENT_FIELD && x.op == TransactionFilterOp::Eq)
                .map(|x| match &x.value {
                    TransactionFilterValue::String(v) => v.as_str(),
                    _ => "",
                })
                .unwrap_or("");

            let deposits = recipient_deposits_from_tx_body(body, recipient).map_err(|_| {
                StdError::generic_err(
                    "sudo_tx_query_result failed to decode recipient deposits from tx body",
                )
            })?;
            // If we didn't find a Send message with the correct recipient, return an error, and
            // this query result will be rejected by Neutron: no data will be saved to state.
            if deposits.is_empty() {
                return Err(StdError::generic_err(
                    "failed to find a matching transaction message",
                ));
            }

            let mut stored_transfers: u64 = TRANSFERS.load(deps.storage).unwrap_or_default();
            stored_transfers += deposits.len() as u64;
            TRANSFERS.save(deps.storage, &stored_transfers)?;

            check_deposits_size(&deposits)?;
            let mut stored_deposits: Vec<Transfer> = RECIPIENT_TXS
                .load(deps.storage, recipient)
                .unwrap_or_default();
            stored_deposits.extend(deposits);
            RECIPIENT_TXS.save(deps.storage, recipient, &stored_deposits)?;
            Ok(Response::new())
        }
    }
}

/// parses tx body and retrieves transactions to the given recipient.
fn recipient_deposits_from_tx_body(
    tx_body: TxBody,
    recipient: &str,
) -> NeutronResult<Vec<Transfer>> {
    let mut deposits: Vec<Transfer> = vec![];
    // Only handle up to MAX_ALLOWED_MESSAGES messages, everything else
    // will be ignored to prevent 'out of gas' conditions.
    // Note: in real contracts you will have to somehow save ignored
    // data in order to handle it later.
    for msg in tx_body.messages.iter().take(MAX_ALLOWED_MESSAGES) {
        // Skip all messages in this transaction that are not Send messages.
        if msg.type_url != *COSMOS_SDK_TRANSFER_MSG_URL.to_string() {
            continue;
        }

        // Parse a Send message and check that it has the required recipient.
        let transfer_msg: MsgSend = MsgSend::decode(msg.value.as_slice())?;
        if transfer_msg.to_address == recipient {
            for coin in transfer_msg.amount {
                deposits.push(Transfer {
                    sender: transfer_msg.from_address.clone(),
                    amount: coin.amount.clone(),
                    denom: coin.denom,
                    recipient: recipient.to_string(),
                });
            }
        }
    }
    Ok(deposits)
}

// checks whether there are deposits that are greater then MAX_ALLOWED_TRANSFER.
fn check_deposits_size(deposits: &Vec<Transfer>) -> StdResult<()> {
    for deposit in deposits {
        match deposit.amount.parse::<u64>() {
            Ok(amount) => {
                if amount > MAX_ALLOWED_TRANSFER {
                    return Err(StdError::generic_err(format!(
                        "maximum allowed transfer is {}",
                        MAX_ALLOWED_TRANSFER
                    )));
                };
            }
            Err(error) => {
                return Err(StdError::generic_err(format!(
                    "failed to cast transfer amount to u64: {}",
                    error
                )));
            }
        };
    }
    Ok(())
}

/// sudo_kv_query_result is the contract's callback for KV query results. Note that only the query
/// id is provided, so you need to read the query result from the state.
pub fn sudo_kv_query_result(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    query_id: u64,
) -> StdResult<Response<NeutronMsg>> {
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_kv_query_result received; query_id: {:?}",
            query_id,
        )
        .as_str(),
    );

    // TODO: provide an actual example. Currently to many things are going to change
    // after @pro0n00gler's PRs to implement this.

    Ok(Response::default())
}
