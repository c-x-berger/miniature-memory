#![feature(try_trait)]

use std::{
    fs::File,
    io,
    io::{BufReader, Read},
    net::{TcpListener, TcpStream},
    os::unix::net::UnixStream,
    path::PathBuf,
    sync::Arc,
    thread,
};

use common::{Network, UpdateMessage};
use dashmap::DashMap;
use structopt::StructOpt;

mod database;
use database::{Database, Record};

const ACCEPTED_PROTO_VERSION: u8 = 66;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long, default_value = "./database", parse(from_os_str))]
    db_file: PathBuf,
}

fn main() -> io::Result<()> {
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
    let db = Arc::new(Database::new(base));
    let worked_db = db.clone();
    thread::spawn(move || {
        println!("Binding to localhost:1515");
        let listener = TcpListener::bind("0.0.0.0:1515").unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let db_handle = worked_db.clone();
            thread::spawn(move || handle(stream, db_handle));
        }
    });
    println!("creating SIGINT handler...");
    let (mut read, write) = UnixStream::pair()?;
    signal_hook::pipe::register(signal_hook::SIGINT, write)?;
    let mut buf = [0];
    read.read_exact(&mut buf)?;
    println!("happy shutdown!");
    Ok(())
}

fn handle(mut client: TcpStream, db: Arc<Database>) -> io::Result<()> {
    println!("handling new connection");
    println!("start parsing message");
    let mut buf: Vec<u8> = Vec::new();
    client.read_to_end(&mut buf)?;
    let message = UpdateMessage::from_networking(&buf)?;
    assert_eq!(message.version(), ACCEPTED_PROTO_VERSION);
    println!(
        "read update message complete!\n- timestamp: {}\n- label: {}\n- value: {}",
        message.timestamp(),
        message.label(),
        message.value()
    );
    if message.correct_signature().is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Signature broken",
        ));
    }
    let label = String::from(message.label());
    db.add_record(&label, Record::from(message));
    Ok(())
}
