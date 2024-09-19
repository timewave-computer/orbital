pub(crate) mod user {
    use cosmwasm_std::{ensure, Env, MessageInfo, Response};

    use crate::{
        contract::{ExecuteDeps, OrbitalResult},
        error::ContractError,
        state::{UserConfig, CLEARING_ACCOUNTS, ORBITAL_DOMAINS, USER_CONFIGS},
    };

    pub fn register_new_domain(
        deps: ExecuteDeps,
        _env: Env,
        info: MessageInfo,
        domain: String,
    ) -> OrbitalResult {
        // user must be registered to operate on domains
        ensure!(
            USER_CONFIGS.has(deps.storage, info.sender.to_string()),
            ContractError::UserNotRegistered {}
        );

        // the domain must be enabled on orbital level to be able to register
        ensure!(
            ORBITAL_DOMAINS.has(deps.storage, domain.to_string()),
            ContractError::UnknownDomain(domain)
        );

        let domain_config = ORBITAL_DOMAINS.load(deps.storage, domain.to_string())?;
        let mut user_config = USER_CONFIGS.load(deps.storage, info.sender.to_string())?;

        // update the registered domains for the caller
        user_config.registered_domains.push(domain.to_string());

        // store `None` as the clearing account until the callback is received
        // from the registration message, which will fill the clearing account
        CLEARING_ACCOUNTS.save(deps.storage, (domain, info.sender.to_string()), &None)?;
        //save the updated user config
        USER_CONFIGS.save(deps.storage, info.sender.to_string(), &user_config)?;

        Ok(Response::new()
            .add_message(domain_config.get_registration_message(deps, &info)?)
            .add_attribute("method", "register_user_domain"))
    }

    pub fn register(deps: ExecuteDeps, _env: Env, info: MessageInfo) -> OrbitalResult {
        // user can only register once
        ensure!(
            !USER_CONFIGS.has(deps.storage, info.sender.to_string()),
            ContractError::UserAlreadyRegistered {}
        );

        // save an empty user config
        USER_CONFIGS.save(
            deps.storage,
            info.sender.to_string(),
            &UserConfig::default(),
        )?;

        Ok(Response::new().add_attribute("method", "register_user"))
    }
}
