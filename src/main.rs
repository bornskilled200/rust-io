#![feature(proc_macro_hygiene)]

use serde::{Serialize, Deserialize};

use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};
use thruster::{App, BasicContext as Ctx, Request};
use thruster::thruster_middleware::send::file;
use thruster::server::Server;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    particles: i32,
    time: i32,
}

#[derive(Debug)]
enum MyError {
    StdIoError(std::io::Error),
}

impl From<std::io::Error> for MyError {
    fn from(e: std::io::Error) -> Self {
        MyError::StdIoError(e)
    }
}

#[middleware_fn]
async fn index(context: Ctx, _next: MiddlewareNext<Ctx>) ->  MiddlewareResult<Ctx> {
    Ok(file(context, "public/index.html"))
}

#[middleware_fn]
async fn stylesheet(context: Ctx, _next: MiddlewareNext<Ctx>) ->  MiddlewareResult<Ctx> {
    Ok(file(context, "public/stylesheets/style.css"))
}

#[middleware_fn]
async fn conditions(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

#[middleware_fn]
async fn four_oh_four(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

fn main() {
    let app: App<Request, Ctx> = {
        let mut app = App::<Request, Ctx>::new_basic();
        app.set404(async_middleware!(Ctx, [four_oh_four]));
        app.get("/", async_middleware!(Ctx, [index]));
        app.get("/stylesheets/style.css", async_middleware!(Ctx, [stylesheet]));
        app
    };

    let server = Server::new(app);
    server.start("0.0.0.0", 8080);
}
