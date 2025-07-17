use log::info;
use serde::Deserialize;

use crate::{control_block::{parse_input, ControlBlock}, db::get_sql_opt, engine::return_code::*, make_failed_resp, make_success_resp};

pub async fn ping(payload: String) -> ReturnCode {
    make_success_resp!(payload: format!("payload: {{{payload}}}"))
}

#[derive(Deserialize)]
pub struct RegisterReq {
    pub user_name: String,
    pub password: String,
}

pub async fn register(payload: String) -> ReturnCode {
    let sql_opt = get_sql_opt().await;
    let (_, content) = match parse_input::<RegisterReq>(&payload) {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    let rst = sql_opt
        .register(&content.user_name, &content.password)
        .await;
    if let Err(e) = rst {
        return make_failed_resp!(payload: e);
    }

    info!("user {} register", content.user_name);

    let block = ControlBlock::from_user_name(&content.user_name);

    make_success_resp!(block: block)
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub user_name: String,
    pub password: String,
}

pub async fn login(payload: String) -> ReturnCode {
    let sql_opt = get_sql_opt().await;
    let (_, content) = match parse_input::<LoginReq>(&payload) {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    let rst = sql_opt
        .login(&content.user_name, &content.password)
        .await;
    if let Err(e) = rst {
        return make_failed_resp!(payload: e);
    }
    match rst {
        Ok(rst) => {
            if !rst {
                return make_failed_resp!(payload: "login failed");
            }
            info!("user {} login", content.user_name);
        },
        Err(e) => {
            return make_failed_resp!(payload: e);
        }
    }

    let block = ControlBlock::from_user_name(&content.user_name);

    make_success_resp!(block: block)
}

pub async fn refresh(payload: String) -> ReturnCode {
    let (mut block, _) = match parse_input::<i32>(&payload) {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    match block.validate_jwt() {
        Ok(rst) => {
            if !rst {
                return make_failed_resp!(payload: "jwt expired");
            } else {
                if let Err(e) = block.refresh_jwt() {
                    return make_failed_resp!(payload: format!("refresh jwt err: {e}"));
                }
            }
        }
        Err(e) => {
            return make_failed_resp!(payload: format!("invalid jwt: {e}"));
        }
    }

    make_success_resp!(block: block)
}