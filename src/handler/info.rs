use serde::{Deserialize, Serialize};

use crate::{control_block::parse_input, db::{get_sql_opt, FileInfo}, engine::return_code::ReturnCode, make_failed_resp, make_success_resp};

#[derive(Deserialize)]
pub struct ListFileReq {
    filter: String
}

#[derive(Serialize)]
pub struct ListFileResp {
    file_info: Vec<FileInfo>,
}

pub async fn list_file(payload: String) -> ReturnCode {
    let (_, req) = match parse_input::<ListFileReq>(&payload) {
        Ok((block, req)) => (block, req),
        Err(e) => return make_failed_resp!(payload: e),
    };

    let sql_opt = get_sql_opt().await;

    let ids = match sql_opt.get_file_ids().await {
        Ok(ids) => ids,
        Err(e) => return make_failed_resp!(payload: e)
    };

    let mut file_info_list = vec![];
    for id in ids {
        let file_info = match sql_opt.get_file_info_by_id(id).await {
            Ok(file_info) => file_info,
            Err(e) => return make_failed_resp!(payload: e)
        };

        if file_info.file_status != 1 {
            continue;
        }

        if file_info.file_name.contains(&req.filter) {
            file_info_list.push(file_info);
        }
    }

    let resp = ListFileResp {
        file_info: file_info_list,
    };

    let resp = match serde_json::to_string(&resp) {
        Ok(resp) => resp,
        Err(e) => return make_failed_resp!(payload: e)
    };

    make_success_resp!(payload: resp)
}

#[derive(Deserialize)]
pub struct DeleteFileReq {
    file_id: i32,
}

pub async fn delete_file(payload: String) -> ReturnCode {
    let (block, req) = match parse_input::<DeleteFileReq>(&payload) {
        Ok((block, req)) => (block, req),
        Err(e) => return make_failed_resp!(payload: e),
    };

    match block.validate_jwt() {
        Ok(rst) => {
            if !rst {
                return make_failed_resp!(payload: "jwt validate failed");
            }
        },
        Err(e) => {
            return make_failed_resp!(payload: e)
        }
    }

    let sql_opt = get_sql_opt().await;

    match sql_opt.delete_file_info(req.file_id).await {
        Ok(_) => make_success_resp!(),
        Err(e) => make_failed_resp!(payload: e)
    }
}

#[derive(Deserialize)]
pub struct GetFileInfoReq {
    file_id: i32,
}

pub async fn get_file_info(payload: String) -> ReturnCode {
    let (_, req) = match parse_input::<GetFileInfoReq>(&payload) {
        Ok((block, req)) => (block, req),
        Err(e) => return make_failed_resp!(payload: e),
    };

    let sql_opt = get_sql_opt().await;

    match sql_opt.get_file_info_by_id(req.file_id).await {
        Ok(file_info) => {
            let resp = match serde_json::to_string(&file_info) {
                Ok(resp) => resp,
                Err(e) => return make_failed_resp!(payload: e)
            };

            make_success_resp!(payload: resp)
        },
        Err(e) => make_failed_resp!(payload: e)
    }
}