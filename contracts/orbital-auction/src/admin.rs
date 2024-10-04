use cosmwasm_std::{ensure, Env, MessageInfo, Response};
use neutron_sdk::{bindings::msg::NeutronMsg, NeutronResult};

use crate::{
    contract::ExecuteDeps,
    error::ContractError,
    state::{UserIntent, ACTIVE_AUCTION, ORBITAL_CORE, ORDERBOOK},
};

/// admin-gated action to include a (validated) user intent into the orderbook.
pub fn enqueue_user_intent(
    deps: ExecuteDeps,
    info: MessageInfo,
    env: Env,
    user_intent: UserIntent,
) -> NeutronResult<Response<NeutronMsg>> {
    // only the admin can enqueue new orders on behalf of the users.
    // this is because orbital-core (the admin) is responsible for
    // escrowing the user's funds and ensuring the auction is fair.
    // this function is called post-validation, so we can trust the input.
    ensure!(
        info.sender == ORBITAL_CORE.load(deps.storage)?,
        ContractError::Unauthorized {}
    );

    // if the current auction is not yet in bidding phase (but ready),
    // and the current batch is not yet full, we try to include this order in the batch directly
    let mut current_auction = ACTIVE_AUCTION.load(deps.storage)?;

    // if the current auction already started the bidding phase, we push the intent to the orderbook
    if current_auction
        .phases
        .start_expiration
        .is_expired(&env.block)
    {
        // add the user intent to the end of the orderbook queue
        ORDERBOOK.push_back(deps.storage, &user_intent)?;
    } else {
        // otherwise, we still have time to include the order in the active batch that is about to start
        // to do that, we first check if the batch has the capacity to fit this order
        if current_auction.batch.can_fit_order(&user_intent) {
            current_auction.batch.include_order(user_intent)?;
            ACTIVE_AUCTION.save(deps.storage, &current_auction)?;
        } else {
            // if the batch is full, we push the intent to the orderbook
            ORDERBOOK.push_back(deps.storage, &user_intent)?;
        }
    }

    Ok(Response::default())
}
