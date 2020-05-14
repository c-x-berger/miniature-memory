#![feature(try_trait)]

use std::{fs::File, io, io::BufReader, path::PathBuf, sync::Arc};

use dashmap::DashMap;
use ed25519_dalek::{PublicKey, Signature};
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

fn invalid_data(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

async fn handle(mut client: TcpStream, db: Arc<database::Database>) -> io::Result<()> {
    println!("handling new connection");
    println!("start parsing message");
    let version = client.read_u8().await?;
    println!("message version: {}", version);
    if version != ACCEPTED_PROTO_VERSION {
        return Err(invalid_data("wrong proto version!"));
    }
    // TODO: check version
    let mut key_bytes: [u8; 32] = [0; 32];
    let mut sig_bytes: [u8; 64] = [0; 64];
    client.read_exact(&mut key_bytes).await?;
    client.read_exact(&mut sig_bytes).await?;
    let key = PublicKey::from_bytes(&key_bytes);
    let sig = Signature::from_bytes(&sig_bytes);
    if key.is_err() || sig.is_err() {
        return Err(invalid_data("error parsing key/sig"));
    }
    // 50 bytes seems a reasonable, but arbitrarty prealloc
    let mut remaining_buf: Vec<u8> = Vec::with_capacity(50);
    client.read_to_end(&mut remaining_buf).await?;
    match key.unwrap().verify(&remaining_buf, &sig.unwrap()) {
        Ok(()) => {}
        Err(_) => return Err(invalid_data("signature broken!")),
    };
    let mut remaining = remaining_buf.as_slice();
    // shh, nobody tell tokio
    let timestamp = remaining.read_u64().await?;
    let label_len = remaining.read_u8().await?;
    let mut label: Vec<u8> = vec![0; usize::from(label_len)];
    remaining.read_exact(&mut label).await?;
    let label = match String::from_utf8(label) {
        Ok(s) => s,
        Err(_) => return Err(invalid_data("label is not valid UTF-8")),
    };
    let mut value = String::new();
    remaining.read_to_string(&mut value).await?;
    println!(
        "read update message complete!\n- timestamp: {}\n- label: {}\n- value: {}",
        timestamp, label, value
    );
    Ok(())
}
