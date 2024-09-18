use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coins, ensure, Addr, MessageInfo, StdResult, Uint128, Uint64};
use cw_storage_plus::Map;
use cw_utils::must_pay;
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    query::min_ibc_fee::query_min_ibc_fee,
};

use crate::{contract::ExecuteDeps, error::ContractError};

/// map of users with their respective configurations
pub const USER_CONFIGS: Map<Addr, UserConfig> = Map::new("user_configs");

/// map of registered remote domains and their configuration
pub const ORBITAL_DOMAINS: Map<String, OrbitalDomainConfig> = Map::new("domains");

/// map of clearing accounts registered with orbital.
/// key is a composite of (domain_identifier, owner_neutron_addr).
/// value is an optional address where:
/// - None: clearing account is being registered and awaiting callback
/// - Some: clearing account has been registered and is ready for use
pub const CLEARING_ACCOUNTS: Map<(String, String), Option<Addr>> = Map::new("clearing_accounts");

#[cw_serde]
#[derive(Default)]
pub struct UserConfig {}

/// remote domain configuration config which supports different types of account implementations.
/// currently supported types:
/// - Polytone: cw-based account implementation that operates via note contract on the origin chain
/// - InterchainAccount: interchain account implementation based on ICS-27
#[cw_serde]
pub enum OrbitalDomainConfig {
    Polytone {
        note: Addr,
        timeout: Uint64,
    },
    InterchainAccount {
        connection_id: String,
        channel_id: String,
        timeout: Uint64,
    },
}

/// assumes that fees are only denominated in untrn and flattens them into a single coin
fn flatten_ibc_fees_amt(fee_response: IbcFee) -> Uint128 {
    fee_response
        .ack_fee
        .iter()
        .chain(fee_response.recv_fee.iter())
        .chain(fee_response.timeout_fee.iter())
        .map(|fee| fee.amount)
        .sum()
}

impl OrbitalDomainConfig {
    pub fn get_registration_message(
        &self,
        deps: ExecuteDeps,
        info: &MessageInfo,
        domain: String,
    ) -> Result<NeutronMsg, ContractError> {
        let msg = match self {
            OrbitalDomainConfig::InterchainAccount { connection_id, .. } => {
                let min_ibc_fee = query_min_ibc_fee(deps.as_ref()).map_err(|e| {
                    ContractError::DomainRegistrationError(format!(
                        "failed to query min ibc fee: {}",
                        e
                    ))
                })?;
                let expected_fee_payment = flatten_ibc_fees_amt(min_ibc_fee.min_fee);

                match must_pay(info, "untrn") {
                    Ok(amt) => ensure!(
                        amt >= expected_fee_payment,
                        ContractError::DomainRegistrationError("insufficient fee".to_string())
                    ),
                    Err(e) => return Err(ContractError::DomainRegistrationError(e.to_string())),
                }

                NeutronMsg::register_interchain_account(
                    connection_id.to_string(),
                    info.sender.to_string(),
                    Some(coins(100_000, "untrn")),
                )
            }
            _ => unimplemented!(),
        };

        // store `None` as the clearing account until the callback is received
        CLEARING_ACCOUNTS.save(deps.storage, (domain, info.sender.to_string()), &None)?;

        Ok(msg)
    }
}
