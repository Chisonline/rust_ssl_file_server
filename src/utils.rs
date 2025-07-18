#[macro_export]
macro_rules! make_success_resp {
    // 明确区分 payload 和 block 的顺序
    (payload: $payload:expr, block: $block:expr) => {
        $crate::make_resp!(true, payload: $payload, block: $block)
    };
    (payload: $payload:expr) => {
        $crate::make_resp!(true, payload: $payload)
    };
    (block: $block:expr) => {
        $crate::make_resp!(true, block: $block)
    };
    () => {
        $crate::make_resp!(true)
    };
}

#[macro_export]
macro_rules! make_failed_resp {
    (payload: $payload:expr, block: $control_block:expr) => {
        $crate::make_resp!(false, payload: $payload, block: $control_block)
    };
    (payload: $payload:expr) => {
        $crate::make_resp!(false, payload: $payload)
    };
    (block: $block:expr) => {
        $crate::make_resp!(false, block: $control_block)
    };
    () => {
        $crate::make_resp!(false)
    };
}

#[macro_export]
macro_rules! make_resp {
    ($success:expr, payload: $payload:expr, block: $block:expr) => {{
        let payload = $payload.to_string();
        let block = $block;
        $crate::engine::return_code::ReturnCode {
            success: $success,
            payload: Some(payload),
            control_block: Some(block),
        }
    }};
    ($success:expr, payload: $payload:expr) => {{
        let payload = format!("{}", $payload);
        $crate::engine::return_code::ReturnCode {
            success: $success,
            payload: Some(payload),
            control_block: None,
        }
    }};
    ($success:expr, block: $block:expr) => {{
        let block = $block;
        $crate::engine::return_code::ReturnCode {
            success: $success,
            payload: None,
            control_block: Some(block),
        }
    }};
    ($success:expr) => {{
        $crate::engine::return_code::ReturnCode {
            success: $success,
            payload: None,
            control_block: None,
        }
    }};
}

pub const END_MARK: &str = "\n\n\n";
