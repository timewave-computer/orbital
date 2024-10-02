use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error(transparent)]
    FeePaymentError(#[from] PaymentError),

    #[error("No bond is posted by sender")]
    NoBondPosted {},

    #[error("auction phase error")]
    AuctionPhaseError {},
}

impl From<ContractError> for NeutronError {
    fn from(value: ContractError) -> Self {
        NeutronError::Std(StdError::generic_err(value.to_string()))
    }
}
