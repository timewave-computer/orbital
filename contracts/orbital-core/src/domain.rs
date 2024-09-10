use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, DepsMut, StdError, StdResult, Uint64};

use crate::state::OrbitalDomainConfig;

#[cw_serde]
pub enum UncheckedDomainConfig {
    Polytone {
        domain: String,
        note: String,
        timeout: Uint64,
    },
}

impl UncheckedDomainConfig {
    pub fn get_key(&self) -> String {
        match self {
            UncheckedDomainConfig::Polytone { domain, .. } => domain.to_string(),
        }
    }

    pub fn validate_to_checked(&self, deps: &DepsMut) -> StdResult<OrbitalDomainConfig> {
        match self {
            UncheckedDomainConfig::Polytone {
                domain,
                note,
                timeout,
            } => {
                // validate the note address on orbital chain
                let note_addr = deps.api.addr_validate(&note)?;

                // ensure that the domain name (key) is non-empty and the timeout is > 0
                ensure!(!domain.is_empty(), StdError::generic_err("empty domain"));
                ensure!(
                    timeout.u64() > 0,
                    StdError::generic_err("timeout must be positive")
                );

                Ok(OrbitalDomainConfig::Polytone {
                    domain: domain.to_string(),
                    note: note_addr,
                    timeout: *timeout,
                })
            }
        }
    }
}
