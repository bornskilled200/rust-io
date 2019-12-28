use serde::{Serialize, Deserialize};
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, Write};
use err_ctx::ResultExt;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;

lazy_static! {
    pub static ref DATA: Mutex<Vec<Condition>> = Mutex::new(Vec::new());
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Condition {
    time: u64,
    uptime: u64,
    air: i64,
}

pub fn load_database() -> Result<(), Box<dyn Error>>{
    let file = File::open("db.json").ctx("open database for read")?;
    let mut data: Vec<Condition> = serde_json::from_reader(BufReader::new(file)).ctx("deseralize")?;
    let mut vector = DATA.lock()?;
    Ok(vector.append(&mut data))
}

lazy_static! {
    static ref START: SystemTime = SystemTime::now();
}
pub fn poll_condition() -> Result<Condition, Box<dyn Error>> {
    let air: i64 = if cfg!(target_os = "windows") {
        1
    } else {
        let output = Command::new("/usr/local/lib/airpi/pms5003-snmp")
            .arg("pm2.5")
            .output()?;
        let string = std::str::from_utf8(&output.stdout)?.trim_end();
        string.parse()?
    };

    let now = SystemTime::now();
    Ok(Condition {
        time: now.duration_since(UNIX_EPOCH).unwrap().as_secs(),
        uptime: now.duration_since(*START).unwrap().as_secs(),
        air,
    })
}

pub fn poll() -> Result<(), Box<dyn Error>> {
    let condition = poll_condition()?;
    let json = DATA.lock().map(|mut vector| {
        if vector.len() > 3000 {
            vector.remove(0);
        }
        vector.push(condition);
        serde_json::to_vec(&*vector).unwrap()
    })?;
    let mut file = File::create("db.json")?;
    Ok(file.write_all(&json)?)
}

pub fn get_conditions_json() -> Result<String, Box<dyn Error>> {
    let data = DATA.lock()?;
    let json = serde_json::to_string(&*data).ctx("Serializing data")?;
    Ok(json)
}