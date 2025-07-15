use serde::{Deserialize, Serialize};

use crate::{control_block::parse_input, db::get_sql_opt, engine::return_code::ReturnCode, make_failed_resp, make_success_resp};

#[derive(Deserialize)]
pub struct GetBlockIdsByFileIdReq {
    file_id: i32,
}

#[derive(Serialize)]
pub struct GetBlockIdsByFileIdResp {
    block_ids: Vec<i32>,
}

pub async fn get_block_ids_by_file_id(payload: String) -> ReturnCode {
    let (_, req) = match parse_input::<GetBlockIdsByFileIdReq>(&payload) {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    let sql_opt = get_sql_opt().await;
    let block_ids = match sql_opt.get_file_block_ids_by_file_id(req.file_id).await {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    let resp = GetBlockIdsByFileIdResp {
        block_ids,
    };

    let resp = serde_json::to_string(&resp).unwrap();

    make_success_resp!(payload: resp)
}


#[derive(Deserialize)]
pub struct GetBlockInfoReq {
    block_id: i32,
}

pub async fn get_block_info(payload: String) -> ReturnCode {
    let (_, req) = match parse_input::<GetBlockInfoReq>(&payload) {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    let sql_opt = get_sql_opt().await;
    let block_info = match sql_opt.get_block_info_by_id(req.block_id).await {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    let resp = serde_json::to_string(&block_info).unwrap();

    make_success_resp!(payload: resp)
}