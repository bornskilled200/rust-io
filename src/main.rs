#[macro_use]
extern crate lazy_static;

use std::time::Duration;
use std::thread;

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

fn start_polling() {
    thread::spawn(|| {
        loop {
            log_error!(poll());

            thread::sleep(Duration::from_secs(60 * 15));
        };
    });
}

fn main() {
    log_error!(load_database());
    start_polling();

    start_server();
}
