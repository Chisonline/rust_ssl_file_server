use crate::engine::ReturnCode;


pub fn ping(payload: &str) -> ReturnCode {
    ReturnCode {
        success: true,
        payload: Some(format!("pong: {payload}")),
        control_block: None
    }
}