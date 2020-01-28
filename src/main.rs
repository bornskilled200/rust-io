#[macro_use]
extern crate lazy_static;

use std::time::Duration;
use tokio::task;
use tokio::time;
use stream_cancel::{Tripwire};

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

fn spawn_polling(tripwire: Tripwire) {
    task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 5));
        tokio::pin!(tripwire);
        loop {
            tokio::select! {
            _tripped = &mut tripwire => { break },
            _ = interval.tick() => {}
        };
            log_error!(poll().await);
        }
    });
}

#[tokio::main]
async fn main() {
    log_error!(load_database().await);
    let (_trigger, tripwire) = Tripwire::new();
    spawn_polling(tripwire);

    start_server().await;
}
