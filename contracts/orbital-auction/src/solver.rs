use cosmwasm_std::{coin, ensure, BankMsg, MessageInfo, Response, Uint128};
use cw_utils::must_pay;
use neutron_sdk::{bindings::msg::NeutronMsg, NeutronResult};

use crate::{
    contract::ExecuteDeps,
    error::ContractError,
    state::{AuctionPhase, Bid, ACTIVE_AUCTION, AUCTION_CONFIG, POSTED_BONDS},
};

/// attempts to place a bid on the active auction round
pub fn try_bid(deps: ExecuteDeps, bid: Bid) -> NeutronResult<Response<NeutronMsg>> {
    // load the existing bond posted by the sender
    let posted_bond = POSTED_BONDS.load(deps.storage, bid.solver.to_string())?;
    let auction_config = AUCTION_CONFIG.load(deps.storage)?;
    let mut active_auction = ACTIVE_AUCTION.load(deps.storage)?;

    // ensure that solver had posted a bond high enough to bid on the auction
    ensure!(
        posted_bond.amount >= auction_config.solver_bond.amount,
        ContractError::BondTooLow {}
    );

    // bids are only accepted during the bidding phase
    ensure!(
        active_auction.phases.get_current_phase(&bid.bid_block) == AuctionPhase::Bidding,
        ContractError::AuctionPhaseError {}
    );

    active_auction.batch.current_bid = match active_auction.batch.current_bid {
        Some(current_bid) => {
            // first we assert that the bid is higher than the current one (and non-zero)
            ensure!(
                bid.amount > current_bid.amount && bid.amount > Uint128::zero(),
                ContractError::BidTooLow {}
            );
            Some(bid)
        }
        None => Some(bid),
    };
    ACTIVE_AUCTION.save(deps.storage, &active_auction)?;

    Ok(Response::default())
}

pub fn try_withdraw_posted_bond(
    deps: ExecuteDeps,
    info: MessageInfo,
) -> NeutronResult<Response<NeutronMsg>> {
    // load the existing bond posted by the sender
    let posted_bond = POSTED_BONDS.load(deps.storage, info.sender.to_string())?;

    // generate a withdraw message
    let withdraw_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![posted_bond],
    };

    // remove the sender entry from posted bonds and transfer the funds
    POSTED_BONDS.remove(deps.storage, info.sender.to_string());

    Ok(Response::default().add_message(withdraw_msg))
}

pub fn try_post_bond(deps: ExecuteDeps, info: MessageInfo) -> NeutronResult<Response<NeutronMsg>> {
    let auction_config = AUCTION_CONFIG.load(deps.storage)?;

    // get the amount of tokens sent by the solver
    let posted_bond_amount = must_pay(&info, &auction_config.solver_bond.denom)
        .map_err(ContractError::FeePaymentError)?;

    // depending on if this is the first time the solver is posting a bond,
    // or if they have already posted a bond before, we return the total
    let new_bond = match POSTED_BONDS.may_load(deps.storage, info.sender.to_string())? {
        Some(existing_bond) => coin(
            existing_bond.amount.checked_add(posted_bond_amount)?.u128(),
            existing_bond.denom,
        ),
        None => coin(posted_bond_amount.u128(), &auction_config.solver_bond.denom),
    };

    POSTED_BONDS.save(deps.storage, info.sender.to_string(), &new_bond)?;

    Ok(Response::default())
}
