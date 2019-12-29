#[macro_use]
extern crate lazy_static;

use std::time::Duration;
use tokio::task;
use tokio::time;
use tokio::prelude::*;

mod sensor;
pub use sensor::{Condition, DATA, load_database, poll, get_conditions_json};

mod server;
pub use server::start_server;

macro_rules! log_error {
    ($exp: expr) => {
        if let Err(err) = $exp {
            println!("{:?}", err);
        }
    };
}

async fn start_polling() {
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        log_error!(poll());
    }
}

#[tokio::main]
async fn main() {
    log_error!(load_database());
    task::spawn(start_polling());

    start_server();
}
