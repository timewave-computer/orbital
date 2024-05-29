use cosmwasm_std::StdError;
use cw_utils::PaymentError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Never {}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Sender is not an admin")]
    NotOwner,

    #[error("Must send funds to start vesting")]
    NoFundsSent,
}
