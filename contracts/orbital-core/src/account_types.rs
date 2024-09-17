use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Api, StdError, StdResult, Uint64};

use crate::state::OrbitalDomainConfig;

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
