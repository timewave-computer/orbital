use cosmwasm_std::{ensure, Coin, DepsMut, MessageInfo, StdResult};
use orbital_utils::intent::Intent;

use crate::{
    state::{next_id, IDS, INTENTS, QUEUE},
    types::Config,
    ContractError,
};

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

/// Ensure the bond is paid and return the bid amount
pub fn get_bid(config: &Config, info: &MessageInfo) -> Result<Coin, ContractError> {
    let bid = if info.funds.len() == 1 {
        let funds = info.funds[0].clone();
        ensure!(
            funds.denom == config.bond_amount.denom,
            ContractError::InvalidDenom(funds.denom)
        );

        Coin {
            denom: funds.denom,
            amount: funds.amount.checked_sub(config.bond_amount.amount)?,
        }
    } else if info.funds.len() == 2 {
        if info.funds[0].denom == config.bond_amount.denom {
            ensure!(
                info.funds[0].denom == config.bond_amount.denom
                    && info.funds[0].amount == config.bond_amount.amount,
                ContractError::InvalidBond(info.funds[1].clone())
            );
            info.funds[1].clone()
        } else {
            ensure!(
                info.funds[1].denom == config.bond_amount.denom
                    && info.funds[1].amount == config.bond_amount.amount,
                ContractError::InvalidBond(info.funds[1].clone())
            );
            info.funds[0].clone()
        }
    } else {
        return Err(ContractError::InvalidBid);
    };
    Ok(bid)
}
