use cosmos_sdk_proto::traits::MessageExt;
use cosmwasm_std::{
    from_json, to_json_binary, Addr, AnyMsg, Api, Binary, BlockInfo, CustomMsg, CustomQuery,
    GrpcQuery, Querier, Storage,
};
use cw_multi_test::{
    error::{anyhow, AnyResult},
    AppResponse, CosmosRouter, Stargate,
};
use neutron_sdk::proto_types::neutron::interchaintxs::v1::{Params, QueryParamsResponse};

use serde::de::DeserializeOwned;

pub struct StargateModule;

impl Stargate for StargateModule {
    fn execute_stargate<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        sender: Addr,
        type_url: String,
        value: Binary,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        cw_multi_test::error::bail!(
            "Unexpected stargate execute in custom StargateModule impl: type_url={}, value={} from {}",
            type_url,
            value,
            sender,
        )
    }

    fn query_stargate(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        path: String,
        data: Binary,
    ) -> AnyResult<Binary> {
        cw_multi_test::error::bail!(
            "Unexpected stargate query in custom StargateModule impl: path={}, data={}",
            path,
            data
        )
    }

    fn execute_any<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        sender: Addr,
        msg: AnyMsg,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        cw_multi_test::error::bail!(
            "Unexpected any execute in custom StargateModule impl: msg={:?} from {}",
            msg,
            sender
        )
    }

    fn query_grpc(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: GrpcQuery,
    ) -> AnyResult<Binary> {
        let query: GrpcQuery = from_json(to_json_binary(&request).unwrap()).unwrap();
        if query.path == "/neutron.interchaintxs.v1.Query/Params" {
            let proto_coin = cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                denom: "untrn".to_string(),
                amount: "1000000".to_string(),
            };

            let params = Params {
                msg_submit_tx_max_messages: 1_000,
                register_fee: vec![proto_coin],
            };

            let response = QueryParamsResponse {
                params: Some(params),
            };

            let binary = Binary::from(response.to_bytes()?);

            return Ok(binary);
        }

        Err(anyhow!("Unexpected query request"))
    }
}

// impl Module for StargateModule {
//     type ExecT = AnyMsg;
//     type QueryT = GrpcQuery;
//     type SudoT = Empty;

//     fn execute<ExecC, QueryC>(
//         &self,
//         _api: &dyn Api,
//         _storage: &mut dyn Storage,
//         _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
//         _block: &BlockInfo,
//         sender: Addr,
//         msg: Self::ExecT,
//     ) -> AnyResult<AppResponse>
//     where
//         ExecC: CustomMsg + DeserializeOwned + 'static,
//         QueryC: CustomQuery + DeserializeOwned + 'static,
//     {
//         Err(anyhow!(
//             "Unexpected exec msg {} from {sender:?}",
//             msg.type_url
//         ))
//     }

//     fn query(
//         &self,
//         _api: &dyn Api,
//         _storage: &dyn Storage,
//         _querier: &dyn Querier,
//         _block: &BlockInfo,
//         _request: Self::QueryT,
//     ) -> AnyResult<Binary> {
//         // println!("custom stargate query request: {:?}", request);

//         // let query: GrpcQuery = from_json(to_json_binary(&request).unwrap()).unwrap();
//         // if query.path == "/neutron.interchaintxs.v1.Query/Params" {
//         //     let params = Params {
//         //         msg_submit_tx_max_messages: Uint64::new(1_000),
//         //         register_fee: coins(1_000_000, "untrn"),
//         //     };

//         //     let response = QueryParamsResponse {
//         //         params: Some(params),
//         //     };

//         //     return Ok(to_json_binary(&response).unwrap());
//         // }

//         Err(anyhow!("Unexpected query request"))
//     }

//     fn sudo<ExecC, QueryC>(
//         &self,
//         _api: &dyn Api,
//         _storage: &mut dyn Storage,
//         _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
//         _block: &BlockInfo,
//         _msg: Self::SudoT,
//     ) -> AnyResult<AppResponse>
//     where
//         ExecC: CustomMsg + DeserializeOwned + 'static,
//         QueryC: CustomQuery + DeserializeOwned + 'static,
//     {
//         println!("custom stargate sudo call: {:?}", _msg);

//         unimplemented!("Stargate sudo is not implemented")
//     }
// }
