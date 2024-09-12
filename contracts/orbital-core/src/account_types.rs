use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, DepsMut, StdError, StdResult, Uint64};

use crate::state::OrbitalDomainConfig;

#[cw_serde]
pub enum AccountConfigType {
    Polytone {
        note: String,
        timeout: Uint64,
    },
    ICA {
        connection_id: String,
        channel_id: String,
        timeout: Uint64,
    },
}

impl AccountConfigType {
    pub fn try_into_domain_config(self, deps: &DepsMut) -> StdResult<OrbitalDomainConfig> {
        match self {
            AccountConfigType::Polytone { note, timeout } => {
                // ensure that the timeout is > 0
                ensure!(
                    timeout.u64() > 0,
                    StdError::generic_err("timeout must be non-zero")
                );

                let validated_config = OrbitalDomainConfig::Polytone {
                    // validate the note address on orbital chain
                    note: deps.api.addr_validate(&note)?,
                    timeout,
                };

                Ok(validated_config)
            }
            AccountConfigType::ICA {
                connection_id,
                channel_id,
                timeout,
            } => {
                // ensure that the timeout is > 0
                ensure!(
                    timeout.u64() > 0,
                    StdError::generic_err("timeout must be non-zero")
                );

                Ok(OrbitalDomainConfig::ICA {
                    connection_id,
                    channel_id,
                    timeout,
                })
            }
        }
    }
}
