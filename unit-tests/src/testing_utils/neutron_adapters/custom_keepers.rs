// use cosmwasm_schema::serde::de::DeserializeOwned;
// use cosmwasm_schema::serde::Serialize;
// use cosmwasm_std::{
//     coin, coins, from_json, to_json_binary, Addr, Api, Binary, BlockInfo, CustomMsg, CustomQuery,
//     GrpcQuery, Querier, Storage, Uint64,
// };
// use cw_multi_test::error::{AnyError, AnyResult};
// use cw_multi_test::{AppResponse, CosmosRouter, Module};
// use orbital_core::state::{Params, QueryParamsResponse};

// use std::fmt::Debug;
// use std::marker::PhantomData;

// pub struct CustomStargateKeeper<ExecT, QueryT, SudoT>(
//     PhantomData<(ExecT, QueryT, SudoT)>,
//     &'static str,
//     &'static str,
//     &'static str,
// );

// impl<ExecT, QueryT, SudoT> CustomStargateKeeper<ExecT, QueryT, SudoT> {
//     pub fn new(execute_msg: &'static str, query_msg: &'static str, sudo_msg: &'static str) -> Self {
//         Self(Default::default(), execute_msg, query_msg, sudo_msg)
//     }
// }

// impl Module for CustomStargateKeeper<ExecT, QueryT, SudoT>
// where
//     ExecT: Debug + Serialize,
//     QueryT: Debug + Serialize,
//     SudoT: Debug,
// {
//     type ExecT = StargateMsg;
//     type QueryT = StargateQuery;
//     type SudoT = Empty;

//     fn execute<ExecC, QueryC>(
//         &self,
//         _api: &dyn Api,
//         _storage: &mut dyn Storage,
//         _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
//         _block: &BlockInfo,
//         _sender: Addr,
//         _msg: Self::ExecT,
//     ) -> AnyResult<AppResponse>
//     where
//         ExecC: CustomMsg + DeserializeOwned + 'static,
//         QueryC: CustomQuery + DeserializeOwned + 'static,
//     {
//         Ok(AppResponse::default())
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
//         Ok(AppResponse::default())
//     }

//     fn query(
//         &self,
//         _api: &dyn Api,
//         _storage: &dyn Storage,
//         _querier: &dyn Querier,
//         _block: &BlockInfo,
//         request: QueryT,
//     ) -> AnyResult<Binary> {
//         println!("custom keeper query request: {:?}", request);

//         let query: GrpcQuery = from_json(to_json_binary(&request).unwrap()).unwrap();
//         if query.path == "/neutron.interchaintxs.v1.Query/Params" {
//             let params = Params {
//                 msg_submit_tx_max_messages: Uint64::new(1_000),
//                 register_fee: coins(1_000_000, "untrn"),
//             };

//             let response = QueryParamsResponse {
//                 params: Some(params),
//             };

//             return Ok(to_json_binary(&response).unwrap());
//         }

//         Err(AnyError::msg(self.2))
//     }
// }
