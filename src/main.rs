#[macro_use]
extern crate lazy_static;

use std::time::Duration;
use tokio::task;
use tokio::time;
use stream_cancel::{Tripwire};
use futures::{select, FutureExt};
use futures::future::Fuse;

mod sensor;
pub use sensor::{Condition, load_database, poll, get_conditions_json};

mod server;
pub use server::start_server;

macro_rules! log_error {
    ($exp: expr) => {
        if let Err(err) = $exp {
            println!("{:?}", err);
        }
    };
}

async fn start_polling(mut tripwire: Fuse<Tripwire>) {
    let mut interval = time::interval(Duration::from_secs(60 * 5));
    loop {
        select! {
            _ = tripwire => break,
            _ = interval.tick().fuse() => {},
        };
        log_error!(poll().await);
    }
}

#[tokio::main]
async fn main() {
    log_error!(load_database().await);
    let (_trigger, tripwire) = Tripwire::new();
    task::spawn(start_polling(tripwire.fuse()));

    start_server().await;
}
