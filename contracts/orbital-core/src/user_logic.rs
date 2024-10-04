pub(crate) mod user {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
    use cosmwasm_std::{ensure, Coin, Env, MessageInfo, Response, StdError, Uint64};
    use cw_utils::must_pay;
    use neutron_sdk::{
        bindings::msg::NeutronMsg, interchain_queries::v047::types::COSMOS_SDK_TRANSFER_MSG_URL,
        query::min_ibc_fee::query_min_ibc_fee, NeutronResult,
    };

    use crate::{
        contract::ExecuteDeps,
        error::ContractError,
        state::{UserConfig, CLEARING_ACCOUNTS, ORBITAL_DOMAINS, USER_CONFIGS, USER_NONCE},
        utils::{fees::flatten_ibc_fees_amt, generate_proto_msg, get_ica_identifier},
    };

    pub fn try_register_new_domain(
        deps: ExecuteDeps,
        _env: Env,
        info: MessageInfo,
        domain: String,
    ) -> NeutronResult<Response<NeutronMsg>> {
        // user must be registered in order to operate on domains
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

        // get the ica identifier
        let ica_identifier = get_ica_identifier(user_config.id, domain.to_string());

        // update the registered domains for the caller
        user_config.registered_domains.push(domain.to_string());

        // store `None` as the clearing account until the callback is received
        // from the registration message, which will fill the clearing account
        CLEARING_ACCOUNTS.save(deps.storage, ica_identifier.to_string(), &None)?;
        //save the updated user config
        USER_CONFIGS.save(deps.storage, info.sender.to_string(), &user_config)?;

        Ok(Response::new()
            .add_message(domain_config.get_registration_message(deps, &info, ica_identifier)?)
            .add_attribute("method", "register_user_domain"))
    }

    pub fn try_register(
        deps: ExecuteDeps,
        _env: Env,
        info: MessageInfo,
    ) -> NeutronResult<Response<NeutronMsg>> {
        // user can only register once
        ensure!(
            !USER_CONFIGS.has(deps.storage, info.sender.to_string()),
            ContractError::UserAlreadyRegistered {}
        );

        let user_nonce = USER_NONCE.load(deps.storage)?;

        // save an empty user config
        USER_CONFIGS.save(
            deps.storage,
            info.sender.to_string(),
            &UserConfig {
                id: user_nonce,
                registered_domains: vec![],
            },
        )?;
        // increment the nonce
        USER_NONCE.save(deps.storage, &user_nonce.checked_add(Uint64::one())?)?;

        Ok(Response::new().add_attribute("method", "register_user"))
    }

    pub fn try_withdraw_from_remote_domain(
        deps: ExecuteDeps,
        info: MessageInfo,
        domain: String,
        coin: Coin,
        dest: String,
    ) -> NeutronResult<Response<NeutronMsg>> {
        let user_config = USER_CONFIGS.load(deps.storage, info.sender.to_string())?;

        // first we validate that user is registered to the domain from which they
        // want to withdraw funds from
        ensure!(
            user_config.registered_domains.contains(&domain),
            ContractError::UserNotRegisteredToDomain(domain)
        );

        // validate that IBC fees are covered by the caller
        let min_ibc_fee = query_min_ibc_fee(deps.as_ref())?;
        let total_fee_amt = flatten_ibc_fees_amt(&min_ibc_fee.min_fee);
        let paid_amt = must_pay(&info, "untrn").map_err(ContractError::FeePaymentError)?;

        ensure!(
            paid_amt >= total_fee_amt,
            ContractError::Std(StdError::generic_err("insufficient fee coverage"))
        );

        // derive the port associated with user's clearing account
        let ica_identifier = get_ica_identifier(user_config.id, domain.to_string());

        let user_clearing_acc_config = CLEARING_ACCOUNTS
            .load(deps.storage, ica_identifier.to_string())?
            .ok_or_else(|| ContractError::UserNotRegisteredToDomain(domain))?;

        // generate the transfer message to be executed on target domain
        let proto_coin = ProtoCoin {
            denom: coin.denom,
            amount: coin.amount.to_string(),
        };
        let bank_msg = cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend {
            from_address: user_clearing_acc_config.addr,
            to_address: dest,
            amount: vec![proto_coin],
        };

        let proto_msg = generate_proto_msg(bank_msg, COSMOS_SDK_TRANSFER_MSG_URL)?;

        let withdraw_tx: NeutronMsg = NeutronMsg::submit_tx(
            user_clearing_acc_config.controller_connection_id,
            ica_identifier,
            vec![proto_msg],
            "".to_string(),
            60,
            min_ibc_fee.min_fee,
        );

        Ok(Response::default().add_message(withdraw_tx))
    }
}
