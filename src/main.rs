#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};
use thruster::{App, BasicContext as Context, Request, map_try};
use thruster::thruster_middleware::send::file;
use thruster::server::Server;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};
use std::time::Duration;
use thruster::errors::ThrusterError;
use std::thread;

mod sensor;
pub use sensor::{Condition, DATA, load_database, poll, get_conditions_json};

macro_rules! log_error {
    ($exp: expr) => {
        if let Err(err) = $exp {
            println!("{:?}", err);
        }
    };
}

macro_rules! simple_try {
    ($exp: expr, $ctx: ident, $message: expr) => {
        simple_try!($exp, $ctx, $message, 500);
    };
    ($exp: expr, $ctx: ident, $message: expr, $status: expr) => {
        map_try!($exp, Err(err) => {
            ThrusterError { context: $ctx, cause: Some(err.into()), message: $message.into(), status: $status }
        });
    };
}

#[middleware_fn]
async fn index(context: Context, _next: MiddlewareNext<Context>) ->  MiddlewareResult<Context> {
    Ok(file(context, "public/index.html"))
}

#[middleware_fn]
async fn stylesheet(context: Context, _next: MiddlewareNext<Context>) ->  MiddlewareResult<Context> {
    Ok(file(context, "public/stylesheets/style.css"))
}

#[middleware_fn]
async fn conditions_handler(mut context: Context, _next: MiddlewareNext<Context>) -> MiddlewareResult<Context> {
    let json = simple_try!(get_conditions_json(), context, "error during get conditions");
    context.body(&json);

    Ok(context)
}

#[middleware_fn]
async fn four_oh_four(mut context: Context, _next: MiddlewareNext<Context>) -> MiddlewareResult<Context> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
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
    let app: App<Request, Context> = {
        let mut app = App::<Request, Context>::new_basic();
        app.set404(async_middleware!(Context, [four_oh_four]));
        app.get("/", async_middleware!(Context, [index]));
        app.get("/stylesheets/style.css", async_middleware!(Context, [stylesheet]));
        app.get("/conditions", async_middleware!(Context, [conditions_handler]));
        app
    };

    let server = Server::new(app);
    server.start("0.0.0.0", 3000);
}
