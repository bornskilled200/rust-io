#[macro_use]
extern crate lazy_static;

mod sensor;
pub use sensor::{Condition, load_database, spawn_poller, get_conditions_json};

mod server;
pub use server::start_server;
use std::sync::Arc;
use tokio::sync::Notify;
use simple_logger::SimpleLogger;
use log::error;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().init().unwrap();
    let notify = Arc::new(Notify::new());

    load_database().await.unwrap_or_else(|e| error!("{:?}", e));
    let poller = spawn_poller(notify.clone());

    // actix-web handles SIGINT (ctrl + c) and SIGTERM
    start_server().await?;
    notify.notify();
    poller.await.unwrap_or_else(|e| error!("{:?}", e));
    Ok(())
}
