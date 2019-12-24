#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use serde::{Serialize, Deserialize};

use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};
use thruster::{App, BasicContext as Ctx, Request, map_try};
use thruster::thruster_middleware::send::file;
use thruster::server::Server;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};
use diesel::{
    prelude::*,
    sqlite::SqliteConnection,
    table,
};
use tokio_diesel::*;
use std::time::{SystemTime, UNIX_EPOCH};
use thruster::errors::ThrusterError;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    time: u128,
    uptime: u128,
    air: f64,
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

#[derive(Queryable, Debug)]
struct Condition {
    id: i32,
    time: i32,
    uptime: i32,
    air: f32,
}

table! {
    conditions {
        id -> Integer,
        time -> Integer,
        uptime -> Integer,
        air -> Float,
    }
}

#[middleware_fn]
async fn conditions_handler(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    use crate::conditions::columns::{id, time, uptime, air};
    context.status(404);
    let connection = map_try!(SqliteConnection::establish("bme2.db"), Err(err) => {
        ThrusterError { context, cause: Some(Box::new(err)), message: "connection".into(), status: 1 }
    });
    let x = map_try!(conditions::table.load::<Condition>(&connection), Err(err) => {
        ThrusterError { context, cause: Some(Box::new(err)), message: "connection".into(), status: 1 }
    });
    context.body(&format!("{:?}", x));


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
        app.get("/conditions", async_middleware!(Ctx, [conditions_handler]));
        app
    };

    let server = Server::new(app);
    server.start("0.0.0.0", 8080);
}
