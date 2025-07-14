use crate::{engine::return_code::*, make_success_resp};

pub async fn ping(payload: String) -> ReturnCode {
    make_success_resp!(payload: format!("payload: {{{payload}}}"))
}

pub struct RegisterReq {
    pub user_name: String,
    pub password: String,
}

pub async fn register(payload: String) -> ReturnCode {
    
    make_success_resp!(payload: "123".to_string())
}