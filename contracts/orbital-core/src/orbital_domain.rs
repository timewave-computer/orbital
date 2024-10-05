use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Api, Coin, MessageInfo, StdError, StdResult, Uint64};
use neutron_sdk::bindings::msg::NeutronMsg;

use crate::{
    contract::ExecuteDeps,
    error::ContractError,
    state::OrbitalDomainConfig,
    utils::fees::{assert_fee_payment, query_ica_registration_fee},
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

impl OrbitalDomainConfig {
    pub fn get_registration_message(
        &self,
        deps: &ExecuteDeps,
        info: &MessageInfo,
        ica_identifier: String,
    ) -> Result<NeutronMsg, ContractError> {
        match self {
            OrbitalDomainConfig::InterchainAccount { connection_id, .. } => {
                let response = query_ica_registration_fee(deps)?;

                // if fee_coins is empty, set value to None; otherwise - set it to Some(fee_coins)
                let registration_fees = if let Some(val) = response.params {
                    let mut fee_coins = vec![];
                    for coin in val.register_fee.iter() {
                        // map from proto coin
                        let fee_coin = Coin {
                            amount: coin.amount,
                            denom: coin.denom.to_string(),
                        };
                        // assert its covered by the sender
                        assert_fee_payment(info, &fee_coin)?;
                        // collect fee coins
                        fee_coins.push(fee_coin);
                    }
                    Some(fee_coins)
                } else {
                    None
                };

                Ok(NeutronMsg::register_interchain_account(
                    connection_id.to_string(),
                    ica_identifier,
                    registration_fees,
                ))
            }
            _ => Err(StdError::generic_err("unimplemented!()").into()),
        }
    }
}
