#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate if_chain;

use serde::{Serialize, Deserialize};

use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};
use thruster::{App, BasicContext as Context, Request, map_try};
use thruster::thruster_middleware::send::file;
use thruster::server::Server;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use thruster::errors::ThrusterError;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::process::Command;
use std::fs::File;
use std::io::{BufReader, Write};

#[derive(Serialize, Deserialize, Debug)]
struct Condition {
    time: u64,
    uptime: u64,
    air: i64,
}

#[middleware_fn]
async fn index(context: Context, _next: MiddlewareNext<Context>) ->  MiddlewareResult<Context> {
    Ok(file(context, "public/index.html"))
}

#[middleware_fn]
async fn stylesheet(context: Context, _next: MiddlewareNext<Context>) ->  MiddlewareResult<Context> {
    Ok(file(context, "public/stylesheets/style.css"))
}

lazy_static! {
    static ref DATA: Mutex<Vec<Condition>> = Mutex::new(Vec::new());
}

#[middleware_fn]
async fn conditions_handler(mut context: Context, _next: MiddlewareNext<Context>) -> MiddlewareResult<Context> {
    let data = map_try!(DATA.lock(), Err(err) => {
            ThrusterError { context, cause: Some(Box::new(err)), message: "lock data".into(), status: 500 }
    });
    let result = map_try!(serde_json::to_string(&*data), Err(err) => {
            ThrusterError { context, cause: Some(Box::new(err)), message: "serialize data".into(), status: 500 }
    });
    context.body(&format!("{}", result));

    Ok(context)
}

#[middleware_fn]
async fn four_oh_four(mut context: Context, _next: MiddlewareNext<Context>) -> MiddlewareResult<Context> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

fn load_database() {
    if_chain! {
        if let Ok(file) = File::open("db.json");
        if let Ok(mut data) = serde_json::from_reader::<BufReader<File>, Vec<Condition>>(BufReader::new(file));
        if let Ok(mut vector) = DATA.lock();
        then {
            vector.append(&mut data);
        }
    }
}

lazy_static! {
    static ref START: SystemTime = SystemTime::now();
}
fn poll_condition() -> Condition {
    let air: i64 = if cfg!(target_os = "windows") {
        1
    } else {
        let output = Command::new("/usr/local/lib/airpi/pms5003-snmp")
            .arg("pm2.5")
            .output()
            .unwrap();
        let string = std::str::from_utf8(&output.stdout).unwrap().trim_end();
        string.parse().unwrap()
    };

    let now = SystemTime::now();
    Condition {
        time: now.duration_since(UNIX_EPOCH).unwrap().as_secs(),
        uptime: now.duration_since(*START).unwrap().as_secs(),
        air,
    }
}

fn main() {
    load_database();
    thread::spawn({
        loop {
            let condition = poll_condition();
            let json: Result<Vec<u8>, std::sync::PoisonError<MutexGuard<Vec<Condition>>>> = DATA.lock().map(|mut vector| {
                if vector.len() > 3000 {
                    vector.remove(0);
                }
                vector.push(condition);
                serde_json::to_vec(&*vector).unwrap()
            });
            if_chain! {
                if let Ok(j) = json;
                if let Ok(mut file) = File::create("db.json");
                then {
                    file.write_all(&j).unwrap();
                }
            }

            thread::sleep(Duration::from_secs(60 * 15));
        };
    });
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
