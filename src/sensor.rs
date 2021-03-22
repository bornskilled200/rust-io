use serde::{Serialize, Deserialize};
use tokio::fs::{File, OpenOptions, rename};
use std::error::Error;
use anyhow::{Context, Result};
use tokio::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Notify};
use std::io::SeekFrom::End;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;
use log::error;

lazy_static! {
    static ref CONDITIONS: RwLock<VecDeque<Condition>> = RwLock::new(VecDeque::new());
    static ref START: SystemTime = SystemTime::now();
}

static MAX_TIME: u64 = 60 * 60 * 24 * 3;
static POLLING_TIME_SECONDS: u64 = 5 * 60;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Condition {
    time: u64,
    uptime: u64,
    air: i64,
}
static DATABASE_PATH: &str = "db.json";

pub async fn load_database() -> Result<()>{
    let start = *START;
    let contents = {
        let mut file: File = File::open(DATABASE_PATH).await.context("open database to read")?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        contents
    };
    match serde_json::from_slice::<Vec<Condition>>(&contents) {
        Err(err) => {
            error!("Unable to deserialize database, moving database. {:?}", err);
            if let Err(e) = rename(DATABASE_PATH, format!("db-{}.json", start.duration_since(UNIX_EPOCH)?.as_secs())).await {
                return Err(e.into());
            }
             Err(err.into())
        }
        Ok(mut vector) => {
            let minimum_time = start.duration_since(UNIX_EPOCH).unwrap().as_secs() - MAX_TIME;
            let start = vector.iter()
                .rposition(|condition| condition.time < minimum_time)
                .unwrap_or(0);
            let mut conditions = CONDITIONS.write().await;
            *conditions = vector.split_off(start).into();
            Ok(())
        }
    }
}

pub fn spawn_poller(notify: Arc<Notify>) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(POLLING_TIME_SECONDS));
        loop {
            tokio::select! {
                _tripped = notify.notified() => { break },
                _ = interval.tick() => {}
            };

            poll().await.unwrap_or_else(|e| error!("{:?}", e));
        }
    })
}

async fn poll_condition() -> Result<Condition, Box<dyn Error>> {
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
    let now = SystemTime::now();
    let minimum_time = now.duration_since(UNIX_EPOCH).unwrap().as_secs() - MAX_TIME;
    let mut vector = CONDITIONS.write().await;
    while let Some(front) = vector.front() {
        if front.time >= minimum_time {
            break;
        }
        vector.pop_front();
    }
    vector.push_back(condition);
}

async fn poll() -> Result<(), Box<dyn Error>> {
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

pub async fn get_conditions_json() -> Result<(Vec<u8>, Option<u64>)> {
    let conditions = CONDITIONS.read().await;
    Ok((
        serde_json::to_vec(&*conditions).context("Serializing data")?,
        conditions.back().map(|condition| condition.time + POLLING_TIME_SECONDS)
    ))
}