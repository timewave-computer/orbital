pub(crate) mod admin {
    use cosmwasm_std::{ensure, Addr, BlockInfo, MessageInfo, Response};
    use cw_ownable::{assert_owner, update_ownership, Action};

    use crate::{
        account_types::UncheckedOrbitalDomainConfig,
        contract::{ExecuteDeps, OrbitalResult},
        error::ContractError,
        state::ORBITAL_DOMAINS,
    };

    pub fn transfer_admin(
        deps: ExecuteDeps,
        block: &BlockInfo,
        sender: &Addr,
        action: Action,
    ) -> OrbitalResult {
        let resp = update_ownership(deps.into_empty(), block, sender, action)
            .map_err(ContractError::Ownership)?;
        Ok(Response::default().add_attributes(resp.into_attributes()))
    }

    pub fn register_new_domain(
        deps: ExecuteDeps,
        info: MessageInfo,
        domain: String,
        account_type: UncheckedOrbitalDomainConfig,
    ) -> OrbitalResult {
        // only the owner can register new domains
        assert_owner(deps.storage, &info.sender).map_err(ContractError::Ownership)?;

        // validate the domain configuration
        let orbital_domain = account_type.try_into_checked(deps.api)?;

        // ensure the domain does not already exist
        ensure!(
            !ORBITAL_DOMAINS.has(deps.storage, domain.to_string()),
            ContractError::OrbitalDomainAlreadyExists(domain.to_string())
        );

        // store the validated domain config in state
        ORBITAL_DOMAINS.save(deps.storage, domain.to_string(), &orbital_domain)?;

        Ok(Response::default()
            .add_attribute("method", "register_new_domain")
            .add_attribute("domain", domain))
    }
}
