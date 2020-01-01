use serde::{Serialize, Deserialize};
use tokio::fs::{File, OpenOptions, rename};
use std::error::Error;
use err_ctx::ResultExt;
use tokio::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use std::io::SeekFrom::End;
use std::collections::VecDeque;
use tokio::prelude::*;

lazy_static! {
    static ref CONDITIONS: Mutex<VecDeque<Condition>> = Mutex::new(VecDeque::new());
    static ref START: SystemTime = SystemTime::now();
}

static MAX_CONDITIONS: usize = 2000;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Condition {
    time: u64,
    uptime: u64,
    air: i64,
}
static DATABASE_PATH: &str = "db.json";

pub async fn load_database() -> Result<(), Box<dyn Error>>{
    let start = *START;
    let mut contents = {
        let mut file: File = File::open(DATABASE_PATH).await.ctx("open database for read")?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        contents
    };
    match serde_json::from_slice::<Vec<Condition>>(&contents) {
        Err(err) => {
            println!("Unable to deserialize database, moving database. {:?}", err);
            if let Err(e) = rename(DATABASE_PATH, format!("db-{}.json", start.duration_since(UNIX_EPOCH)?.as_secs())).await {
                return Err(e.into());
            }
             Err(err.into())
        }
        Ok(mut vector) => {
            let len = vector.len();
            let start = if len > MAX_CONDITIONS {
                len - MAX_CONDITIONS
            } else {
                0
            };
            let drain = vector.drain(start..);
            let mut conditions = CONDITIONS.lock().await;
            conditions.reserve(drain.len());
            Ok(conditions.extend(drain))
        }
    }
}

pub async fn poll_condition() -> Result<Condition, Box<dyn Error>> {
    let air: i64 = if cfg!(any(target_os = "windows", target_os = "windows")) {
        1
    } else {
        let output = Command::new("/usr/local/lib/airpi/pms5003-snmp")
            .arg("pm2.5")
            .output()
            .await?;
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

async fn push_condition(condition: Condition) {
    let mut vector = CONDITIONS.lock().await;
    if vector.len() > MAX_CONDITIONS {
        vector.pop_front();
    }
    vector.push_back(condition);
}

pub async fn poll() -> Result<(), Box<dyn Error>> {
    let condition = poll_condition().await?;
    let json = serde_json::to_string(&condition)?;
    push_condition(condition).await;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(DATABASE_PATH)
        .await?;
    let original_size = file.metadata().await?.len();
    let first_char = if original_size == 0 {
        '['
    } else {
        file.seek(End(-("]".len() as i64))).await?;
        ','
    };
    file.write_all(format!("{}{}]", first_char, json).as_bytes()).await?;
    Ok(())
}

pub async fn get_conditions_json() -> Result<Vec<u8>, Box<dyn Error>> {
    let conditions = CONDITIONS.lock().await;
    Ok(serde_json::to_vec(&*conditions).ctx("Serializing data")?)
}