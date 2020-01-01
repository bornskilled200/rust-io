use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult, Context};
use thruster::{App, BasicContext, Request, map_try};
use thruster::thruster_middleware::send::file;
use thruster::server::Server;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};
use thruster::errors::ThrusterError;

use crate::sensor::get_conditions_json;

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
async fn index(context: BasicContext, _next: MiddlewareNext<BasicContext>) ->  MiddlewareResult<BasicContext> {
    Ok(file(context, "public/index.html"))
}

#[middleware_fn]
async fn stylesheet(context: BasicContext, _next: MiddlewareNext<BasicContext>) ->  MiddlewareResult<BasicContext> {
    Ok(file(context, "public/stylesheets/style.css"))
}

#[middleware_fn]
async fn conditions_handler(mut context: BasicContext, _next: MiddlewareNext<BasicContext>) -> MiddlewareResult<BasicContext> {
    let json = simple_try!(get_conditions_json().await, context, "error during get conditions");
    context.set_body(json);

    Ok(context)
}

#[middleware_fn]
async fn four_oh_four(mut context: BasicContext, _next: MiddlewareNext<BasicContext>) -> MiddlewareResult<BasicContext> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

pub fn create_app() -> App<Request, BasicContext> {
    let mut app = App::<Request, BasicContext>::new_basic();
    app.set404(async_middleware!(BasicContext, [four_oh_four]));
    app.get("/", async_middleware!(BasicContext, [index]));
    app.get("/stylesheets/style.css", async_middleware!(BasicContext, [stylesheet]));
    app.get("/conditions", async_middleware!(BasicContext, [conditions_handler]));
    app
}

pub async fn start_server() {
    let server = Server::new(create_app());
    server.build("0.0.0.0", 3000).await;
}
