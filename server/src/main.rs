#![feature(try_trait)]

use std::{fs::File, io, io::BufReader, path::PathBuf, sync::Arc};

use common::{Network, UpdateMessage};
use dashmap::DashMap;
use structopt::StructOpt;
use tokio::{
    net::{TcpListener, TcpStream},
    prelude::*,
};

mod database;

const ACCEPTED_PROTO_VERSION: u8 = 66;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long, default_value = "./database", parse(from_os_str))]
    db_file: PathBuf,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Hello, world!");
    let opts = Options::from_args();
    println!("Loading database from {:?}...", opts.db_file);
    let base = match File::open(&opts.db_file) {
        Ok(file) => {
            // TODO: this should use something better than JSON
            serde_json::from_reader(BufReader::new(file))?
        }
        Err(e) => {
            let base = DashMap::new();
            if e.kind() == io::ErrorKind::NotFound {
                println!("creating new db file...");
                File::create(&opts.db_file)?;
            }
            base
        }
    };
    let db = database::Database::new(base);
    let db = Arc::new(db);
    println!("Binding to localhost:1515");
    let mut listener = TcpListener::bind("0.0.0.0:1515").await?;
    loop {
        let (socket, _addr) = listener.accept().await?;
        let db_handle = db.clone();
        tokio::spawn(async move { handle(socket, db_handle).await });
    }
}

async fn handle(mut client: TcpStream, db: Arc<database::Database>) -> io::Result<()> {
    println!("handling new connection");
    println!("start parsing message");
    let mut buf: Vec<u8> = Vec::new();
    client.read_to_end(&mut buf).await?;
    let message = UpdateMessage::from_networking(&mut buf)?;
    assert_eq!(message.version(), ACCEPTED_PROTO_VERSION);
    println!(
        "read update message complete!\n- timestamp: {}\n- label: {}\n- value: {}",
        message.timestamp(),
        message.label(),
        message.value()
    );
    Ok(())
}
