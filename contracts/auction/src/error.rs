use cosmwasm_std::{Coin, DecimalRangeExceeded, OverflowError, StdError, Uint128};
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

    #[error(transparent)]
    OverflowError(#[from] OverflowError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Sender is not an admin")]
    NotOwner,

    #[error("Must send funds to start vesting")]
    NoFundsSent,

    #[error("Auction is expired")]
    AuctionExpired,

    #[error("Invalid denom was provided: {0}")]
    InvalidDenom(String),

    #[error("Invalid bond was provided: {0}")]
    InvalidBond(Coin),

    #[error("Invalid funds sent with the bid")]
    InvalidBid,

    #[error("Bid too low: minimum of {0} required (without bond)")]
    BidTooLow(Uint128),

    #[error("No intent to run auction for")]
    IntentNotFound,

    #[error("No bids were made on this auction")]
    NoBids,

    #[error("Current intent wasn't found in the queue")]
    QueueIsEmpty,

    #[error("Bond mismatch: {0}")]
    BondMismatch(Coin),
}
