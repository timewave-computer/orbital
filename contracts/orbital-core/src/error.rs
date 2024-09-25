use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error(transparent)]
    FeePaymentError(#[from] PaymentError),

    #[error("Orbital domain already registered: {0}")]
    OrbitalDomainAlreadyExists(String),

    #[error("User already registered")]
    UserAlreadyRegistered {},

    #[error("User not registered")]
    UserNotRegistered {},

    #[error("Unknown domain: {0}")]
    UnknownDomain(String),

    #[error("Domain registration error: {0}")]
    DomainRegistrationError(String),
}

impl From<ContractError> for NeutronError {
    fn from(value: ContractError) -> Self {
        NeutronError::Std(StdError::generic_err(value.to_string()))
    }
}
