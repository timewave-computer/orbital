use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, DepsMut, StdError, StdResult, Uint64};

use crate::state::OrbitalDomainConfig;

#[cw_serde]
pub enum UncheckedOrbitalDomainConfig {
    Polytone { note: String, timeout: Uint64 },
}

impl UncheckedOrbitalDomainConfig {
    pub fn try_into_checked(&self, deps: &DepsMut) -> StdResult<OrbitalDomainConfig> {
        match self {
            UncheckedOrbitalDomainConfig::Polytone { note, timeout } => {
                // ensure that the timeout is > 0
                ensure!(
                    timeout.u64() > 0,
                    StdError::generic_err("timeout must be non-zero")
                );

                let validated_config = OrbitalDomainConfig::Polytone {
                    // validate the note address on orbital chain
                    note: deps.api.addr_validate(note)?,
                    timeout: *timeout,
                };

                Ok(validated_config)
            }
        }
    }
}
