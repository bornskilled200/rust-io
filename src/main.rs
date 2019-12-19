use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{Write, Read, Seek};
use std::io::SeekFrom;
use futures::executor;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    particles: i32,
    time: i32,
}

#[derive(Debug)]
enum MyError {
    StdIoError(std::io::Error),
    BincodeError(Box<bincode::ErrorKind>),
}

impl From<std::io::Error> for MyError {
    fn from(e: std::io::Error) -> Self {
        MyError::StdIoError(e)
    }
}

impl From<Box<bincode::ErrorKind>> for MyError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        MyError::BincodeError(e)
    }
}

async fn create_file() -> Result<File, std::io::Error> {
    return OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("a.txt")
}

async fn deserialize(mut file: &File) -> Result<Vec<Data>, MyError>{
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    return bincode::deserialize(&mut contents).map_err(|e| MyError::from(e));
}

async fn serialize(mut file: &File, datas: Vec<Data>) -> Result<(), MyError> {
    let encoded = bincode::serialize(&datas).map_err(|e| MyError::from(e))?;
    println!("{:?}", encoded);
    file.write_all(&encoded)?;
    Ok(())
}

async fn say_hello() -> Result<bool, MyError> {
    println!("creating file");
    let file: File = create_file().await?;
    println!("created file");

    let mut datas = deserialize(&file).await?;
    println!("deserialized");

    datas.extend(vec![Data { particles: 1, time: 2 }, Data { particles: 3, time: 4 }]);
    println!("extended {:?}", datas);

    serialize(&file, datas).await?;

    return Ok(true);
}

fn main() -> Result<(), MyError> {
    println!("start");
    executor::block_on(say_hello())?;
    println!("end");
    Ok(())
}
