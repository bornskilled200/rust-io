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
    pub static ref DATA: Mutex<VecDeque<Condition>> = Mutex::new(VecDeque::new());
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Condition {
    time: u64,
    uptime: u64,
    air: i64,
}
static DATABASE_PATH: &str = "db.json";

pub async fn load_database() -> Result<(), Box<dyn Error>>{
    let mut file: File = File::open(DATABASE_PATH).await.ctx("open database for read")?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    match serde_json::from_slice(&contents) {
        Err(err) => {
            println!("Unable to deserialize database, moving database. {:?}", err);
            if let Err(e) = rename(DATABASE_PATH, "db2.json").await {
                return Err(e.into());
            }
             Err(err.into())
        }
        Ok(mut data) => {
            let mut vector = DATA.lock().await;
            Ok(vector.append(&mut data))
        }
    }
}

lazy_static! {
    static ref START: SystemTime = SystemTime::now();
}
pub async fn poll_condition() -> Result<Condition, Box<dyn Error>> {
    let air: i64 = if cfg!(target_os = "windows") {
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

pub async fn poll() -> Result<(), Box<dyn Error>> {
    let condition = poll_condition().await?;
    let json = serde_json::to_string(&condition)?;
    let mut vector = DATA.lock().await;
    if vector.len() > 3000 {
        vector.pop_front();
    }
    vector.push_back(condition);
    drop(vector);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(DATABASE_PATH)
        .await?;
    let original_size = file.metadata().await?.len();
    let first_char;
    if original_size == 0 {
        first_char = '[';
    } else {
        file.seek(End(-1)).await?;
        first_char = ',';
    }
    file.write_all(format!("{}{}]", first_char, json).as_bytes()).await?;
    Ok(())
}

pub async fn get_conditions_json() -> Result<String, Box<dyn Error>> {
    let data = DATA.lock().await;
    let json = serde_json::to_string(&*data).ctx("Serializing data")?;
    Ok(json)
}