pub(crate) mod admin {
    use cosmwasm_std::{
        ensure, instantiate2_address, to_json_binary, to_json_string, Addr, BlockInfo,
        CodeInfoResponse, CosmosMsg, Env, MessageInfo, Response, StdError, StdResult, Uint64,
        WasmMsg,
    };
    use cw_ownable::{assert_owner, update_ownership, Action};
    use neutron_sdk::{bindings::msg::NeutronMsg, NeutronResult};
    use orbital_common::msg_types::OrbitalAuctionInstantiateMsg;

    use crate::{
        contract::ExecuteDeps,
        error::ContractError,
        orbital_domain::UncheckedOrbitalDomainConfig,
        state::{
            OrbitalAuctionConfig, ORBITAL_AUCTIONS, ORBITAL_AUCTION_CODE_ID, ORBITAL_AUCTION_NONCE,
            ORBITAL_DOMAINS, ORBITAL_ROUTE_TO_AUCTION_ID,
        },
        utils::ClearingIcaIdentifier,
    };

    pub fn try_get_instantiate_auction_msg(
        deps: ExecuteDeps,
        env: Env,
        ica_identifier: String,
        auction_id: u64,
    ) -> StdResult<CosmosMsg<NeutronMsg>> {
        let mut associated_orbital_auction = ORBITAL_AUCTIONS.load(deps.storage, auction_id)?;
        let code_id = ORBITAL_AUCTION_CODE_ID.load(deps.storage)?;

        let salt = ica_identifier.as_bytes();

        let core_contract = compute_auction_address(&deps, code_id.u64(), &env, salt)?;
        associated_orbital_auction.auction_addr = Some(core_contract.to_string());

        let instantiate_call: CosmosMsg<NeutronMsg> = WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: code_id.u64(),
            label: ica_identifier.to_string(),
            msg: to_json_binary(&associated_orbital_auction.orbital_auction_instantiate_msg)?,
            funds: vec![],
            salt: salt.into(),
        }
        .into();

        ORBITAL_AUCTIONS.save(deps.storage, auction_id, &associated_orbital_auction)?;

        Ok(instantiate_call)
    }

    fn compute_auction_address(
        deps: &ExecuteDeps,
        code_id: u64,
        env: &Env,
        salt: &[u8],
    ) -> StdResult<Addr> {
        let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;

        let canonical_self_address = deps.api.addr_canonicalize(env.contract.address.as_str())?;

        instantiate2_address(checksum.as_slice(), &canonical_self_address, salt)
            .map_err(|_| StdError::generic_err("Failed to get instantiate2 addr"))
            .and_then(|addr| deps.api.addr_humanize(&addr))
    }

    pub fn try_register_new_auction(
        deps: ExecuteDeps,
        info: MessageInfo,
        instantiate_msg: OrbitalAuctionInstantiateMsg,
    ) -> NeutronResult<Response<NeutronMsg>> {
        // only the owner can register new auctions
        assert_owner(deps.storage, &info.sender).map_err(ContractError::Ownership)?;

        let auction_id = ORBITAL_AUCTION_NONCE.load(deps.storage)?;

        let src_domain_ica_identifier = ClearingIcaIdentifier::Auction {
            auction_id: auction_id.u64(),
            domain: instantiate_msg.route_config.src_domain.to_string(),
        }
        .to_str_identifier();
        let dest_domain_ica_identifier = ClearingIcaIdentifier::Auction {
            auction_id: auction_id.u64(),
            domain: instantiate_msg.route_config.dest_domain.to_string(),
        }
        .to_str_identifier();
        let instantiate_src_clearing_acc_msg = get_auction_clearing_ica_registration_msg(
            &deps,
            &info,
            instantiate_msg.route_config.src_domain.to_string(),
            src_domain_ica_identifier.to_string(),
        )?;
        let instantiate_dest_clearing_acc_msg = get_auction_clearing_ica_registration_msg(
            &deps,
            &info,
            instantiate_msg.route_config.dest_domain.to_string(),
            dest_domain_ica_identifier.to_string(),
        )?;
        ORBITAL_ROUTE_TO_AUCTION_ID.save(
            deps.storage,
            to_json_string(&instantiate_msg.route_config)?,
            &auction_id,
        )?;
        ORBITAL_AUCTION_NONCE.save(deps.storage, &auction_id.checked_add(Uint64::one())?)?;
        ORBITAL_AUCTIONS.save(
            deps.storage,
            auction_id.u64(),
            &OrbitalAuctionConfig {
                src_domain: instantiate_msg.route_config.src_domain.to_string(),
                src_clearing_acc_id: src_domain_ica_identifier,
                src_clearing_acc_addr: None,
                dest_domain: instantiate_msg.route_config.dest_domain.to_string(),
                dest_clearing_acc_id: dest_domain_ica_identifier,
                dest_clearing_acc_addr: None,
                orbital_auction_instantiate_msg: instantiate_msg,
                auction_addr: None,
            },
        )?;

        // here we fire the clearing account registration messages.
        // on callback, they will register into the stored `OrbitalAuctionConfig`.
        // once both are registered, the sudo callback handler will instantiate
        // the auction contract.
        Ok(Response::default()
            .add_message(instantiate_src_clearing_acc_msg)
            .add_message(instantiate_dest_clearing_acc_msg))
    }

    fn get_auction_clearing_ica_registration_msg(
        deps: &ExecuteDeps,
        info: &MessageInfo,
        domain: String,
        ica_identifier: String,
    ) -> Result<NeutronMsg, ContractError> {
        let domain_config = ORBITAL_DOMAINS.load(deps.storage, domain.to_string())?;

        let instantiate_auction_clearing_acc_msg =
            domain_config.get_registration_message(deps, info, ica_identifier)?;

        Ok(instantiate_auction_clearing_acc_msg)
    }

    pub fn try_update_ownership(
        deps: ExecuteDeps,
        block: &BlockInfo,
        sender: &Addr,
        action: Action,
    ) -> NeutronResult<Response<NeutronMsg>> {
        let resp = update_ownership(deps.into_empty(), block, sender, action)
            .map_err(ContractError::Ownership)?;
        Ok(Response::default().add_attributes(resp.into_attributes()))
    }

    pub fn try_register_new_domain(
        deps: ExecuteDeps,
        info: MessageInfo,
        domain: String,
        account_type: UncheckedOrbitalDomainConfig,
    ) -> NeutronResult<Response<NeutronMsg>> {
        // only the owner can register new domains
        assert_owner(deps.storage, &info.sender).map_err(ContractError::Ownership)?;

        // validate the domain configuration
        let orbital_domain = account_type.try_into_checked(deps.api)?;

        // ensure the domain does not already exist
        ensure!(
            !ORBITAL_DOMAINS.has(deps.storage, domain.to_string()),
            ContractError::OrbitalDomainAlreadyExists(domain.to_string())
        );

        // TODO: ensure that the domain identifier is fine to use as part of
        // ica identifier?

        // store the validated domain config in state
        ORBITAL_DOMAINS.save(deps.storage, domain.to_string(), &orbital_domain)?;

        Ok(Response::default()
            .add_attribute("method", "register_new_domain")
            .add_attribute("domain", domain))
    }
}
