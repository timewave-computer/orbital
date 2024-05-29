use cosmwasm_std::{DepsMut, StdResult};
use orbital_utils::intent::Intent;

use crate::state::{next_id, IDS, INTENTS, QUEUE};

/// Add the intent to out list and the queue
pub fn add_intent(deps: DepsMut, intent: Intent) -> StdResult<()> {
    let id = next_id(deps.as_ref().storage)?;

    IDS.save(deps.storage, &id)?;
    // add to our intent list
    INTENTS.save(deps.storage, id, &intent)?;
    // add to the queue
    QUEUE.enqueue(deps.storage, id)?;

    Ok(())
}

// remove intent from our system
pub fn remove_intent(deps: DepsMut) -> StdResult<Option<u64>> {
    Ok(match QUEUE.dequeue(deps.storage)? {
        Some(id) => {
            if id > 0 {
                INTENTS.remove(deps.storage, id - 1);
            }

            Some(id)
        }
        None => None,
    })
}
