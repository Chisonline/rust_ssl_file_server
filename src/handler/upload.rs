use crate::{
    control_block::parse_input,
    db::get_sql_opt,
    engine::return_code::ReturnCode,
    make_failed_resp, make_success_resp,
};
use crc::{CRC_32_ISO_HDLC, Crc};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

#[derive(Deserialize)]
struct PresendReq {
    pub file_name: String,
    pub file_size: u64,
}

pub async fn presend(payload: String) -> ReturnCode {
    let (control_block, content) = match parse_input::<PresendReq>(&payload) {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    match control_block.validate_jwt() {
        Ok(rst) => {
            if !rst {
                return make_failed_resp!(payload: "jwt expired");
            }
        }
        Err(e) => {
            return make_failed_resp!(payload: format!("invalid jwt: {e}"));
        }
    }

    let file_name = &content.file_name;
    let file_size = content.file_size;

    let sql_opt = get_sql_opt().await;

    let file_id = match sql_opt
        .init_file_info(file_name, file_size)
        .await
    {
        Err(e) => return make_failed_resp!(payload: e),
        Ok(id) => id,
    };

    make_success_resp!(payload: file_id)
}

#[derive(Deserialize)]
struct SendReq {
    pub file_id: u32,
    pub block_id: u64,
    pub block_checksum: u32,
    pub block_payload: Vec<u8>,
}

fn make_block_name(file_id: u32, block_id: u64) -> String {
    let uuid = Uuid::new_v4();
    format!("./storage/{}-{}_{}", file_id, block_id, uuid)
}

pub async fn send(payload: String) -> ReturnCode {
    let (control_block, content) = match parse_input::<SendReq>(&payload) {
        Ok(rst) => rst,
        Err(e) => {
            return make_failed_resp!(payload: e);
        }
    };

    match control_block.validate_jwt() {
        Ok(rst) => {
            if !rst {
                return make_failed_resp!(payload: "jwt expired");
            }
        }
        Err(e) => {
            return make_failed_resp!(payload: format!("invalid jwt: {e}"));
        }
    }

    let file_id = content.file_id;
    let block_checksum = content.block_checksum;
    let block_id = content.block_id;
    let block_payload = content.block_payload;

    let block_name = make_block_name(file_id, block_id);

    if block_checksum != checksum(&block_payload) {
        return make_failed_resp!(payload: "wrong checksum");
    }

    let sql_opt = get_sql_opt().await;

    let mut file = match tokio::fs::File::create(&block_name).await
    {
        Ok(f) => f,
        Err(e) => return make_failed_resp!(payload: e),
    };

    if let Err(e) = file.write_all(&block_payload).await {
        return make_failed_resp!(payload: format!("write file err: {e}"));
    }

    if let Err(e) = sql_opt
        .write_block_info(
            file_id,
            block_id,
            &block_name,
            block_payload.len() as u32,
            block_checksum,
        )
        .await
    {
        return make_failed_resp!(payload: e);
    }

    make_success_resp!()
}

fn checksum(payload: &[u8]) -> u32 {
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    crc.checksum(payload)
}

#[derive(Deserialize)]
struct FinishReq {
    pub file_id: u32,
    pub file_checksum: u32
}

pub async fn finish(payload: String) -> ReturnCode {
    let (control_block, content) = match parse_input::<FinishReq>(&payload) {
        Ok(rst) => rst,
        Err(e) => return make_failed_resp!(payload: e),
    };

    match control_block.validate_jwt() {
        Ok(rst) => {
            if !rst {
                return make_failed_resp!(payload: "jwt expired");
            }
        }
        Err(e) => {
            return make_failed_resp!(payload: format!("invalid jwt: {e}"));
        }
    }

    let file_id = content.file_id;
    let file_checksum = content.file_checksum;

    let sql_opt = get_sql_opt().await;

    if let Err(e) = sql_opt.finish_file_info(file_id, file_checksum).await {
        return make_failed_resp!(payload: e);
    }

    make_success_resp!()
}