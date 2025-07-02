use crate::engine::{Engine, SyncHandler};
use crate::log::{log_init, test_log};
use ::log::error;

mod engine;
mod file;
mod log;
mod db;
mod control_block;
mod handler;

#[tokio::main]
async fn main()  {
    log_init();
    test_log();

    let rst = Engine::new()
        .register("ping", SyncHandler(handler::ping))
        .register("file", SyncHandler(file::send))
        .run().await;
    if let Err(e) = rst {
        error!("{}", e);
    }
}
