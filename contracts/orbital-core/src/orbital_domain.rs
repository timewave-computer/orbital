use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    ensure, Api, Binary, Coin, MessageInfo, QueryRequest, StdError, StdResult, Uint128, Uint64,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

use crate::{
    contract::ExecuteDeps, error::ContractError, state::OrbitalDomainConfig,
    utils::assert_fee_payment,
};

#[cw_serde]
pub enum UncheckedOrbitalDomainConfig {
    Polytone {
        note: String,
        timeout: Uint64,
    },
    InterchainAccount {
        connection_id: String,
        channel_id: String,
        timeout: Uint64,
    },
}

impl UncheckedOrbitalDomainConfig {
    pub fn try_into_checked(self, api: &dyn Api) -> StdResult<OrbitalDomainConfig> {
        match self {
            UncheckedOrbitalDomainConfig::Polytone { note, timeout } => {
                // ensure that the timeout is > 0
                ensure!(
                    timeout.u64() > 0,
                    StdError::generic_err("timeout must be non-zero")
                );

                let validated_config = OrbitalDomainConfig::Polytone {
                    // validate the note address on orbital chain
                    note: api.addr_validate(&note)?,
                    timeout,
                };

                Ok(validated_config)
            }
            UncheckedOrbitalDomainConfig::InterchainAccount {
                connection_id,
                channel_id,
                timeout,
            } => {
                // ensure that the timeout is > 0
                ensure!(
                    timeout.u64() > 0,
                    StdError::generic_err("timeout must be non-zero")
                );

                Ok(OrbitalDomainConfig::InterchainAccount {
                    connection_id,
                    channel_id,
                    timeout,
                })
            }
        }
    }
}

#[cw_serde]
struct Params {
    pub msg_submit_tx_max_messages: Uint64,
    pub register_fee: Vec<Coin>,
}

// TODO: remove these in favor for neutron-sdk proto types later
#[cw_serde]
struct QueryParamsResponseCustom {
    pub params: Option<Params>,
}

impl OrbitalDomainConfig {
    pub fn get_registration_message(
        &self,
        deps: ExecuteDeps,
        info: &MessageInfo,
        ica_identifier: String,
    ) -> Result<NeutronMsg, ContractError> {
        match self {
            OrbitalDomainConfig::InterchainAccount { connection_id, .. } => {
                // TODO: remove this explicit allow
                #[allow(deprecated)]
                let stargate_query_msg: QueryRequest<NeutronQuery> = QueryRequest::Stargate {
                    path: "/neutron.interchaintxs.v1.Query/Params".to_string(),
                    data: Binary::new(Vec::new()),
                };

                let response: QueryParamsResponseCustom =
                    deps.querier.query(&stargate_query_msg)?;

                // if fee_coins is empty, set value to None; otherwise - set it to Some(fee_coins)
                let registration_fees = match response.params {
                    Some(val) => {
                        let mut fee_coins = vec![];
                        for coin in val.register_fee.iter() {
                            // map from proto coin
                            let fee_coin = Coin {
                                amount: Uint128::from_str(&coin.amount.to_string())?,
                                denom: coin.denom.to_string(),
                            };
                            // assert its covered by the sender
                            assert_fee_payment(info, &fee_coin)?;
                            // collect fee coins
                            fee_coins.push(fee_coin);
                        }
                        Some(fee_coins)
                    }
                    None => None,
                };

                Ok(NeutronMsg::register_interchain_account(
                    connection_id.to_string(),
                    ica_identifier,
                    registration_fees,
                ))
            }
            _ => unimplemented!(),
        }
    }
}
