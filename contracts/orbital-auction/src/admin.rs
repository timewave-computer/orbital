use cosmwasm_std::{ensure, MessageInfo, Response};
use neutron_sdk::{bindings::msg::NeutronMsg, NeutronResult};

use crate::{
    contract::ExecuteDeps,
    error::ContractError,
    state::{UserIntent, ORBITAL_CORE, ORDERBOOK},
};

/// admin-gated action to include a (validated) user intent into the orderbook.
pub fn enqueue_user_intent(
    deps: ExecuteDeps,
    info: MessageInfo,
    user_intent: UserIntent,
) -> NeutronResult<Response<NeutronMsg>> {
    // only the admin can enqueue new orders on behalf of the users
    ensure!(
        info.sender == ORBITAL_CORE.load(deps.storage)?,
        ContractError::Unauthorized {}
    );

    // add the user intent to the end of the orderbook queue
    ORDERBOOK.push_back(deps.storage, &user_intent)?;

    // TODO: if this order is the only order in the orderbook &
    // active auction is not yet started (finalized & due to start),
    // try to push this order into the active auction for faster inclusion.

    Ok(Response::default())
}