use crate::{engine::engine::Engine, log::log_init};
use ::log::error;
use handler::{upload, user, info, download};

mod engine;
mod handler;
mod log;
mod db;
mod control_block;

#[macro_use]
mod utils;

#[tokio::main]
async fn main()  {
    log_init();

    let rst = Engine::new()
        .set_private_key_file("ssl/key.pem")
        .set_cert_file("ssl/cert.pem")
        .set_port(17878)
        .register("ping", user::ping)
        .register("send", upload::send)
        .register("presend", upload::presend)
        .register("finish", upload::finish)
        .register("register", user::register)   
        .register("login", user::login)
        .register("refresh", user::refresh)
        .register("list_file", info::list_file)
        .register("delete_file", info::delete_file)
        .register("get_block_ids", download::get_block_ids_by_file_id)
        .register("get_block", download::get_block)
        .run().await;

    if let Err(e) = rst {
        error!("{}", e);
    }
}
