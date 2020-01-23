use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult, Context};
use thruster::{App, BasicHyperContext, HyperRequest, map_try};
use thruster::thruster_context::basic_hyper_context::generate_context;
use thruster::hyper_server::HyperServer;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};
use thruster::errors::ThrusterError;
use tokio::fs::File;
use tokio::io::{BufReader, AsyncReadExt};

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

async fn file(mut context: BasicHyperContext, file_name: &str) -> MiddlewareResult<BasicHyperContext> {
    let file = simple_try!(File::open(file_name).await, context, "opening file");
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();

    let _ = buf_reader.read_to_end(&mut contents).await;

    context.set_body(contents);
    Ok(context)
}

#[middleware_fn]
async fn index(context: BasicHyperContext, _next: MiddlewareNext<BasicHyperContext>) ->  MiddlewareResult<BasicHyperContext> {
    file(context, "public/index.html").await
}

#[middleware_fn]
async fn stylesheet(context: BasicHyperContext, _next: MiddlewareNext<BasicHyperContext>) ->  MiddlewareResult<BasicHyperContext> {
    file(context, "public/stylesheets/style.css").await
}

#[middleware_fn]
async fn conditions_handler(mut context: BasicHyperContext, _next: MiddlewareNext<BasicHyperContext>) -> MiddlewareResult<BasicHyperContext> {
    let json = simple_try!(get_conditions_json().await, context, "error during get conditions");
    context.set_body(json);

    Ok(context)
}

#[middleware_fn]
async fn four_oh_four(mut context: BasicHyperContext, _next: MiddlewareNext<BasicHyperContext>) -> MiddlewareResult<BasicHyperContext> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

pub fn create_app() -> App<HyperRequest, BasicHyperContext> {
    let mut app = App::<HyperRequest, BasicHyperContext>::create(generate_context);
    app.set404(async_middleware!(BasicHyperContext, [four_oh_four]));
    app.get("/", async_middleware!(BasicHyperContext, [index]));
    app.get("/stylesheets/style.css", async_middleware!(BasicHyperContext, [stylesheet]));
    app.get("/conditions", async_middleware!(BasicHyperContext, [conditions_handler]));
    app
}

pub async fn start_server() {
    let server = HyperServer::new(create_app());
    server.build("0.0.0.0", 3000).await;
}
