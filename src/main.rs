#[macro_use]
extern crate lazy_static;

mod sensor;
pub use sensor::{Condition, load_database, spawn_polling, get_conditions_json};

mod server;
pub use server::start_server;
use std::sync::Arc;
use tokio::sync::Notify;

macro_rules! log_error {
    ($exp: expr) => {
        if let Err(err) = $exp {
            println!("{:?}", err);
        }
    };
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let notify = Arc::new(Notify::new());

    log_error!(load_database().await);
    let poller = spawn_polling(notify.clone());

    // actix-web handles sigint (ctrl + c)
    start_server().await?;
    notify.notify_one();
    log_error!(poller.await);
    Ok(())
}
