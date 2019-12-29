use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions, rename};
use std::error::Error;
use std::io::{BufReader, Write, Seek};
use err_ctx::ResultExt;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use std::io::SeekFrom::End;
use std::collections::VecDeque;

lazy_static! {
    pub static ref DATA: Mutex<VecDeque<Condition>> = Mutex::new(VecDeque::new());
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Condition {
    time: u64,
    uptime: u64,
    air: i64,
}
static DATABASE_PATH: &str = "db.json";

pub fn load_database() -> Result<(), Box<dyn Error>>{
    let file = File::open(DATABASE_PATH).ctx("open database for read")?;
    let mut data: VecDeque<Condition> = serde_json::from_reader(BufReader::new(file))
        .map_err(|e| -> Box<dyn Error> {
            println!("Unable to deserialize database, moving database. {:?}", e);
            if let Err(err) = rename(DATABASE_PATH, "db2.json") {
                return Box::new(err)
            }
            Box::new(e)
        })?;
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
    let json = serde_json::to_string(&condition)?;
    DATA.lock().map(|mut vector| {
        if vector.len() > 3000 {
            vector.pop_front();
        }
        vector.push_back(condition);
    })?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(DATABASE_PATH)?;
    let original_size = file.metadata()?.len();
    let first_char;
    if original_size == 0 {
        first_char = '[';
    } else {
        file.seek(End(-1))?;
        first_char = ',';
    }
    write!(file, "{}{}]", first_char, json)?;
    Ok(())
}

pub fn get_conditions_json() -> Result<String, Box<dyn Error>> {
    let data = DATA.lock()?;
    let json = serde_json::to_string(&*data).ctx("Serializing data")?;
    Ok(json)
}