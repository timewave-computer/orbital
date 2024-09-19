use std::fmt::Display;

use cosmwasm_std::{Binary, CosmosMsg, Deps, DepsMut, Empty, Response, SubMsg};
use cw_multi_test::{Contract, ContractWrapper};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

/// Turn a neutron response into an empty response
/// This is fine because the contract return an empty response, but our testing enviroment expects a neutron response
/// the contract that uses this function will never emit a neutron response anyways
pub fn execute_into_neutron<E: Display>(
    into: Result<Response, E>,
) -> Result<Response<NeutronMsg>, E> {
    into.map(|r| {
        let mut res: Response<NeutronMsg> = Response::<NeutronMsg>::default();
        res.data = r.data;
        res.messages = r
            .messages
            .into_iter()
            .filter_map(|m| {
                let msg: Option<CosmosMsg<NeutronMsg>> = if let CosmosMsg::Custom(_) = m.msg {
                    Some(CosmosMsg::<NeutronMsg>::Custom(
                        NeutronMsg::RemoveSchedule {
                            name: "".to_string(),
                        },
                    ))
                } else {
                    m.msg.change_custom()
                };

                msg.map(|msg| SubMsg::<NeutronMsg> {
                    id: m.id,
                    msg,
                    gas_limit: m.gas_limit,
                    reply_on: m.reply_on,
                    payload: Binary::default(),
                })
            })
            .collect();
        res.attributes = r.attributes;
        res
    })
}

/// Turn neutron DepsMut into empty DepsMut
pub fn get_empty_depsmut(deps: DepsMut<NeutronQuery>) -> DepsMut<'_, Empty> {
    DepsMut {
        storage: deps.storage,
        api: deps.api,
        querier: deps.querier.into_empty(),
    }
}

/// Turn neutron Deps into empty Deps
pub fn get_empty_deps(deps: Deps<NeutronQuery>) -> Deps<'_, Empty> {
    Deps {
        storage: deps.storage,
        api: deps.api,
        querier: deps.querier.into_empty(),
    }
}

pub fn orbital_core_contract() -> Box<dyn Contract<NeutronMsg, NeutronQuery>> {
    let contract = ContractWrapper::new(
        orbital_core::contract::execute,
        orbital_core::contract::instantiate,
        orbital_core::contract::query,
    )
    .with_reply(orbital_core::contract::reply)
    .with_sudo(orbital_core::contract::sudo)
    .with_migrate(orbital_core::contract::migrate);

    Box::new(contract)
}
