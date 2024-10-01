use cosmwasm_std::{ensure, MessageInfo, Response};
use neutron_sdk::{bindings::msg::NeutronMsg, NeutronResult};

use crate::{
    contract::ExecuteDeps,
    error::ContractError,
    state::{UserIntent, ADMIN, ORDERBOOK},
};

/// admin-gated action to include a (validated) user intent into the orderbook.
pub fn enqueue_user_intent(
    deps: ExecuteDeps,
    info: MessageInfo,
    user_intent: UserIntent,
) -> NeutronResult<Response<NeutronMsg>> {
    // only the admin can enqueue new orders on behalf of the users
    ensure!(
        info.sender == ADMIN.load(deps.storage)?,
        ContractError::Unauthorized {}
    );

    // add the user intent to the end of the orderbook queue
    ORDERBOOK.push_back(deps.storage, &user_intent)?;

    Ok(Response::default())
}
