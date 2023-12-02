//#![allow(unused)]
#[macro_use]
extern crate log;
pub const CRATE_NAME: &str = module_path!();
mod engine;
mod logger;
mod math;
mod rpc;

#[tokio::main]
async fn main() {
    logger::init();
    engine::run().await;
}
