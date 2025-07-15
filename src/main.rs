use crate::{engine::engine::Engine, log::log_init};
use ::log::error;
use handler::{file, user};

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
        .register("send", file::send)
        .register("presend", file::presend)
        .register("finish", file::finish)
        .register("register", user::register)   
        .register("login", user::login)
        .run().await;
    if let Err(e) = rst {
        error!("{}", e);
    }
}
