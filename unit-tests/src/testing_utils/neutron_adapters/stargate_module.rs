use cosmwasm_std::{
    coins, from_json, to_json_binary, Addr, AnyMsg, Api, Binary, BlockInfo, CustomMsg, CustomQuery,
    Empty, GrpcQuery, Querier, Storage, Uint64,
};
use cw_multi_test::{
    error::{anyhow, AnyError, AnyResult},
    AppResponse, CosmosRouter, Module, Stargate,
};
use orbital_core::state::{Params, QueryParamsResponse};
use serde::de::DeserializeOwned;

// use crate::testing_utils::types::{StargateMsg, StargateQuery};

#[derive(Default)]
pub struct StargateModule;

impl Stargate for StargateModule {}

impl Module for StargateModule {
    type ExecT = AnyMsg;
    type QueryT = GrpcQuery;
    type SudoT = Empty;

    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: Self::ExecT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        println!("custom stargate exec call: {:?}", msg);

        match msg.type_url.as_str() {
            _ => Err(anyhow!(
                "Unexpected exec msg {} from {sender:?}",
                msg.type_url
            )),
        }
    }

    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: Self::QueryT,
    ) -> AnyResult<Binary> {
        println!("custom stargate query request: {:?}", request);

        let query: GrpcQuery = from_json(to_json_binary(&request).unwrap()).unwrap();
        if query.path == "/neutron.interchaintxs.v1.Query/Params" {
            let params = Params {
                msg_submit_tx_max_messages: Uint64::new(1_000),
                register_fee: coins(1_000_000, "untrn"),
            };

            let response = QueryParamsResponse {
                params: Some(params),
            };

            return Ok(to_json_binary(&response).unwrap());
        }

        Err(anyhow!("Unexpected query request"))
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _msg: Self::SudoT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        println!("custom stargate sudo call: {:?}", _msg);

        unimplemented!("Stargate sudo is not implemented")
    }
}
