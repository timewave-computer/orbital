use cosmwasm_std::{
    coin, to_json_binary, to_json_string, Addr, Api, BlockInfo, CustomMsg, CustomQuery, StdError,
    StdResult, Storage,
};
use cw_multi_test::{
    error::{bail, AnyError, AnyResult},
    AppResponse, CosmosRouter, MockApiBech32, Module, WasmSudo,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_txs::helpers::get_port_id,
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::SudoMsg,
};

use serde::de::DeserializeOwned;

use crate::testing_utils::{
    consts::{
        ACCOUNTS, DENOM_NTRN, LOCAL_CHANNELS, LOCAL_CHANNELS_VALUES, REMOTE_CHANNELS,
        REMOTE_CHANNELS_VALUES,
    },
    types::OpenAckVersion,
};

pub trait Neutron: Module<ExecT = NeutronMsg, QueryT = NeutronQuery, SudoT = SudoMsg> {}

pub struct NeutronKeeper {
    api: MockApiBech32,
    account_timeout: bool,
}

impl Neutron for NeutronKeeper {}

impl NeutronKeeper {
    pub fn new(prefix: &'static str) -> Self {
        Self {
            api: MockApiBech32::new(prefix),
            account_timeout: false,
        }
    }

    /// Sets our timeout flag, so the next message will return a timeout response instead of a successful response
    pub fn set_timeout(&mut self, timeout: bool) {
        self.account_timeout = timeout;
    }

    pub fn add_local_channel(
        &mut self,
        storage: &mut dyn Storage,
        source_channel: &str,
        other_channel: &str,
    ) -> Result<(), StdError> {
        LOCAL_CHANNELS.save(
            storage,
            source_channel.to_string(),
            &other_channel.to_string(),
        )?;
        LOCAL_CHANNELS_VALUES.save(
            storage,
            other_channel.to_string(),
            &source_channel.to_string(),
        )?;
        Ok(())
    }

    pub fn add_remote_channel(
        &mut self,
        storage: &mut dyn Storage,
        some_channel: &str,
        other_channel: &str,
    ) -> Result<(), StdError> {
        REMOTE_CHANNELS.save(
            storage,
            some_channel.to_string(),
            &other_channel.to_string(),
        )?;
        REMOTE_CHANNELS_VALUES.save(
            storage,
            other_channel.to_string(),
            &some_channel.to_string(),
        )?;
        Ok(())
    }
}

impl NeutronKeeper {
    fn register_account(
        &self,
        storage: &mut dyn Storage,
        sender: Addr,
        conn_id: String,
        account_id: String,
    ) -> Result<(), AnyError> {
        if ACCOUNTS.has(storage, (&sender, conn_id.clone(), account_id.clone())) {
            bail!("Account already registered");
        }

        let addr = self
            .api
            .addr_make(format!("{sender}_{conn_id}_{account_id}").as_str());

        ACCOUNTS
            .save(
                storage,
                (&sender, conn_id.clone(), account_id.clone()),
                &addr,
            )
            .unwrap();
        Ok(())
    }

    fn _remove_account(
        &self,
        storage: &mut dyn Storage,
        sender: &Addr,
        conn_id: String,
        account_id: String,
    ) {
        ACCOUNTS.remove(storage, (sender, conn_id, account_id))
    }

    fn get_account(
        &self,
        storage: &dyn Storage,
        sender: &Addr,
        conn_id: &str,
        account_id: &str,
    ) -> StdResult<Addr> {
        ACCOUNTS.load(
            storage,
            (sender, conn_id.to_string(), account_id.to_string()),
        )
    }
}

impl Module for NeutronKeeper {
    type ExecT = NeutronMsg;
    type QueryT = NeutronQuery;
    type SudoT = SudoMsg;

    /// Currently we only implement register ICA and ibcTransfer and SubmitTx,
    /// maybe we should implement other stuff as well?
    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: Self::ExecT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        println!("neutron keeper execute call: {:?}", msg);
        match msg {
            NeutronMsg::RegisterInterchainAccount {
                connection_id,
                interchain_account_id,
                register_fee,
            } => {
                // Send fees to fee burner
                // we do it mainly to make sure fees are deducted in our tests
                let fee = match register_fee {
                    Some(fee) => fee,
                    None => bail!("No register fee specified"),
                };

                let fee_msg = cosmwasm_std::BankMsg::Burn { amount: fee };

                router.execute(api, storage, block, sender.clone(), fee_msg.into())?;

                // Save the account in our storage for later use
                self.register_account(
                    storage,
                    sender.clone(),
                    connection_id.clone(),
                    interchain_account_id.clone(),
                )?;

                // Complete the registration by calling the sudo entry on the contract
                router.sudo(
                    api,
                    storage,
                    block,
                    cw_multi_test::SudoMsg::Wasm(WasmSudo {
                        contract_addr: sender.clone(),
                        message: to_json_binary(&neutron_sdk::sudo::msg::SudoMsg::OpenAck {
                            port_id: get_port_id(sender.to_string(), interchain_account_id.clone()),
                            channel_id: "channel-1".to_string(),
                            counterparty_channel_id: "channel-1".to_string(),
                            counterparty_version: to_json_string(&OpenAckVersion {
                                version: "ica".to_string(),
                                controller_connection_id: connection_id.clone(),
                                host_connection_id: connection_id.clone(),
                                address: self
                                    .api
                                    .addr_make(
                                        format!("{sender}_{connection_id}_{interchain_account_id}")
                                            .as_str(),
                                    )
                                    .to_string(),
                                encoding: "encoding".to_string(),
                                tx_type: "tx_type".to_string(),
                            })
                            .unwrap(),
                        })
                        .unwrap(),
                    }),
                )?;

                Ok(AppResponse::default())
            }
            _ => {
                println!("custom module execute catch-all arm");
                unimplemented!()
            }
        }
    }

    fn query(
        &self,
        _api: &dyn cosmwasm_std::Api,
        storage: &dyn cosmwasm_std::Storage,
        _querier: &dyn cosmwasm_std::Querier,
        _block: &cosmwasm_std::BlockInfo,
        request: Self::QueryT,
    ) -> cw_multi_test::error::AnyResult<cosmwasm_std::Binary> {
        println!("query request in custom neutron mod: {:?}", request);
        match request {
            NeutronQuery::InterchainAccountAddress {
                owner_address,
                interchain_account_id,
                connection_id,
            } => Ok(to_json_binary(
                &self
                    .get_account(
                        storage,
                        &Addr::unchecked(owner_address),
                        connection_id.as_str(),
                        interchain_account_id.as_str(),
                    )
                    .unwrap(),
            )
            .unwrap()),
            NeutronQuery::MinIbcFee {} => Ok(to_json_binary(&MinIbcFeeResponse {
                min_fee: neutron_sdk::bindings::msg::IbcFee {
                    recv_fee: vec![],
                    ack_fee: vec![coin(10_000, DENOM_NTRN)],
                    timeout_fee: vec![coin(10_000, DENOM_NTRN)],
                },
            })
            .unwrap()),
            _ => {
                println!("custom module query catch-all arm");
                unimplemented!()
            }
        }
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn cosmwasm_std::Api,
        _storage: &mut dyn cosmwasm_std::Storage,
        _router: &dyn cw_multi_test::CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &cosmwasm_std::BlockInfo,
        _msg: Self::SudoT,
    ) -> cw_multi_test::error::AnyResult<cw_multi_test::AppResponse>
    where
        ExecC: std::fmt::Debug
            + Clone
            + PartialEq
            + cosmwasm_schema::schemars::JsonSchema
            + cosmwasm_schema::serde::de::DeserializeOwned
            + 'static,
        QueryC: cosmwasm_std::CustomQuery + cosmwasm_schema::serde::de::DeserializeOwned + 'static,
    {
        bail!("No sudo messages")
    }
}
