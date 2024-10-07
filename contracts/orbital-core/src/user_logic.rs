pub(crate) mod user {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
    use cosmwasm_std::{
        coin, ensure, to_json_string, Coin, Env, MessageInfo, Response, StdError, Uint64,
    };
    use cw_utils::must_pay;
    use neutron_sdk::{
        bindings::{msg::NeutronMsg, types::ProtobufAny},
        interchain_queries::{
            v047::types::COSMOS_SDK_TRANSFER_MSG_URL,
        },
        query::min_ibc_fee::query_min_ibc_fee,
        NeutronResult,
    };

    use crate::{
        contract::ExecuteDeps,
        error::ContractError,
        msg::SubmitIntentMsg,
        state::{
            UserConfig, CLEARING_ACCOUNTS, ORBITAL_AUCTIONS, ORBITAL_DOMAINS,
            ORBITAL_ROUTE_TO_AUCTION_ID, USER_CONFIGS, USER_NONCE,
        },
        utils::{fees::flatten_ibc_fees_amt, generate_proto_msg, ClearingIcaIdentifier},
    };

    /// processes the user-submitted intent before submitting it to the auction.
    pub fn try_submit_intent(
        deps: ExecuteDeps,
        _env: Env,
        info: MessageInfo,
        msg: SubmitIntentMsg,
    ) -> NeutronResult<Response<NeutronMsg>> {
        // first we load all relevant information from the state
        let auction_id =
            ORBITAL_ROUTE_TO_AUCTION_ID.load(deps.storage, to_json_string(&msg.route_config)?)?;
        let auction_config = ORBITAL_AUCTIONS.load(deps.storage, auction_id.u64())?;
        let user_config = USER_CONFIGS.load(deps.storage, info.sender.to_string())?;
        let user_ica_identifier = ClearingIcaIdentifier::User {
            user_id: user_config.id.u64(),
            domain: msg.route_config.src_domain,
        };
        let src_domain_user_clearing_acc = CLEARING_ACCOUNTS
            .load(deps.storage, user_ica_identifier.to_str_identifier())?
            .ok_or(StdError::generic_err(
                "failed to load user clearing account",
            ))?;

        // we need to escrow the offer amount from the user and send it
        // to the clearing account associated with the auction
        // responsible for this route
        let src_domain_auction_deposit_addr =
            auction_config
                .src_clearing_acc_addr
                .ok_or(StdError::generic_err(
                    "auction route not ready for deposits yet",
                ))?;

        // 2. register an ICQ for planned transfer of funds to
        // the auction clearing account
        // let icq_message = register_remote_domain_escrow_tx_query(
        //     &src_domain_auction_deposit_addr,
        //     &src_domain_user_clearing_acc.addr,
        //     &src_domain_user_clearing_acc.controller_connection_id,
        //     5,
        //     msg.amount.to_string(),
        // )?;
        // let icq_message = new_register_transfers_query_msg(
        //     src_domain_user_clearing_acc
        //         .controller_connection_id
        //         .to_string(),
        //     src_domain_auction_deposit_addr.to_string(),
        //     5,
        //     None,
        // )?;

        // 3. submit ICA transfer to the clearing account
        let proto_bank_send_msg = generate_proto_msg_send(
            coin(msg.amount.u128(), msg.route_config.offer_denom),
            src_domain_user_clearing_acc.addr,
            src_domain_auction_deposit_addr,
        )?;

        let min_ibc_fee = query_min_ibc_fee(deps.as_ref())?;
        let total_fee_amt = flatten_ibc_fees_amt(&min_ibc_fee.min_fee);
        let paid_amt = must_pay(&info, "untrn").map_err(ContractError::FeePaymentError)?;

        ensure!(
            paid_amt >= total_fee_amt,
            ContractError::Std(StdError::generic_err("insufficient fee coverage"))
        );

        let escrow_tx: NeutronMsg = NeutronMsg::submit_tx(
            src_domain_user_clearing_acc.controller_connection_id,
            user_ica_identifier.to_str_identifier(),
            vec![proto_bank_send_msg],
            "".to_string(),
            60,
            min_ibc_fee.min_fee,
        );
        // 4. receive ICQ callback, remove the registered ICQ,
        // and submit the intent to the auction

        Ok(Response::default()
            // .add_message(icq_message)
            .add_message(escrow_tx))
    }

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
        let user_ica_identifier = ClearingIcaIdentifier::User {
            user_id: user_config.id.u64(),
            domain: domain.to_string(),
        };

        let ica_identifier_str = user_ica_identifier.to_str_identifier();

        // update the registered domains for the caller
        user_config.registered_domains.push(domain.to_string());

        // store `None` as the clearing account until the callback is received
        // from the registration message, which will fill the clearing account
        CLEARING_ACCOUNTS.save(deps.storage, ica_identifier_str.to_string(), &None)?;
        //save the updated user config
        USER_CONFIGS.save(deps.storage, info.sender.to_string(), &user_config)?;

        Ok(Response::new()
            .add_message(domain_config.get_registration_message(
                &deps,
                &info,
                ica_identifier_str,
            )?)
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
        let user_ica_identifier = ClearingIcaIdentifier::User {
            user_id: user_config.id.u64(),
            domain: domain.to_string(),
        };

        let ica_identifier = user_ica_identifier.to_str_identifier();

        let user_clearing_acc_config = CLEARING_ACCOUNTS
            .load(deps.storage, ica_identifier.to_string())?
            .ok_or_else(|| ContractError::UserNotRegisteredToDomain(domain))?;

        // generate the transfer message to be executed on target domain
        let proto_bank_send_msg =
            generate_proto_msg_send(coin, user_clearing_acc_config.addr, dest)?;

        let withdraw_tx: NeutronMsg = NeutronMsg::submit_tx(
            user_clearing_acc_config.controller_connection_id,
            ica_identifier,
            vec![proto_bank_send_msg],
            "".to_string(),
            60,
            min_ibc_fee.min_fee,
        );

        Ok(Response::default().add_message(withdraw_tx))
    }

    fn generate_proto_msg_send(
        coin: Coin,
        from_addr: String,
        to_addr: String,
    ) -> NeutronResult<ProtobufAny> {
        // generate the transfer message to be executed on target domain
        let proto_coin = ProtoCoin {
            denom: coin.denom,
            amount: coin.amount.to_string(),
        };
        let bank_msg = cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend {
            from_address: from_addr,
            to_address: to_addr,
            amount: vec![proto_coin],
        };

        let proto_msg = generate_proto_msg(bank_msg, COSMOS_SDK_TRANSFER_MSG_URL)?;

        Ok(proto_msg)
    }
}
