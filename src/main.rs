#[macro_use]
extern crate lazy_static;

use stream_cancel::Tripwire;

mod sensor;
pub use sensor::{Condition, load_database, spawn_polling, get_conditions_json};

mod server;
pub use server::start_server;

macro_rules! log_error {
    ($exp: expr) => {
        if let Err(err) = $exp {
            println!("{:?}", err);
        }
    };
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_trigger, tripwire) = Tripwire::new();

    log_error!(load_database().await);
    spawn_polling(tripwire);

    start_server().await?;
    Ok(())
}
