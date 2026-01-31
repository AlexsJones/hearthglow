pub mod cli;
pub mod data;
pub(crate) mod entity;
pub mod server;
use std::env;

#[tokio::main]
async fn main() {
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    cli::run().await;
}
