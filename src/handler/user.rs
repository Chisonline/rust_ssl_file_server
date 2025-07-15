use serde::Deserialize;

use crate::{control_block::{issue_jwt, parse_input, ControlBlock}, db::get_sql_opt, engine::return_code::*, make_failed_resp, make_success_resp};

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

    let block = ControlBlock {
        jwt: issue_jwt(&content.user_name).unwrap(),
    };

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

    let block = ControlBlock {
        jwt: issue_jwt(&content.user_name).unwrap(),
    };

    make_success_resp!(block: block)
}