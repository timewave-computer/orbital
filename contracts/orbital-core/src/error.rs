use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error("Orbital domain already registered: {0}")]
    OrbitalDomainAlreadyExists(String),

    #[error("User already registered")]
    UserAlreadyRegistered {},

    #[error("User not registered")]
    UserNotRegistered {},

    #[error("Unknown domain: {0}")]
    UnknownDomain(String),
}
