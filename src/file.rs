use crate::{control_block::{validate_jwt, ControlBlock}, engine::ReturnCode};
use crc::{Crc, CRC_32_ISO_HDLC};
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize)]
struct FileContentBlock {
    pub file_name: String,
    pub file_checksum: u32,
    pub file_transferred: usize,
    pub block_checksum: u32,
    pub block_payload: String
}

pub fn send(payload: &str) -> ReturnCode {
    let parts: Vec<&str> = payload.split(' ').collect();
    if parts.len() != 3 {
        return ReturnCode {
            success: false,
            payload: Some("invalid params".to_string()),
            control_block: None
        };
    }

    let control_block = parts[1];
    let payload = parts[2];

    let mut control_block: ControlBlock = match serde_json::from_str(control_block) {
        Ok(control_block) => control_block,
        Err(e) => {
            return ReturnCode {
                success: false,
                payload: Some(format!("invalid control block: {}", e)),
                control_block: None
            };
        }
    };

    match control_block.validate_jwt() {
        Ok(rst) => {
            if !rst {
                return ReturnCode {
                    success: false,
                    payload: Some("jwt expired".to_string()),
                    control_block: Some(control_block)
                }
            }
        },
        Err(e) => {
            return ReturnCode {
                success: false,
                payload: Some(format!("invalid jwt: {}", e)),
                control_block: Some(control_block)
            }
        }
    }

    let content_block: FileContentBlock = match serde_json::from_str(payload) {
        Ok(content_block) => content_block,
        Err(e) => {
            return ReturnCode {
                success: false,
                payload: Some(format!("invalid payload: {}", e)),
                control_block: Some(control_block)
            }
        }
    };

    let file_name = content_block.file_name;
    let file_transferred = content_block.file_transferred;
    let block_checksum = content_block.block_checksum;

    if block_checksum != checksum(payload) {
        return ReturnCode {
            success: false,
            payload: Some("错误的checksum".to_string()),
            control_block: Some(control_block)
        };
    }

    let mut file = if file_transferred == 0 {
        std::fs::File::create(&file_name).unwrap()
    } else {
        std::fs::OpenOptions::new()
            .append(true)
            .open(&file_name)
            .unwrap()
    };

    file.write_all(payload.as_bytes()).unwrap();

    ReturnCode {
        success: true,
        payload: None,
        control_block: Some(control_block)
    }
}

fn checksum(payload: &str) -> u32 {
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    crc.checksum(payload.as_bytes())
}
