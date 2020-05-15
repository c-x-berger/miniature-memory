#![feature(try_trait)]
#![feature(duration_constants)]

use std::{
    fs::File,
    io,
    io::{BufReader, Read},
    net::SocketAddr,
    path::PathBuf,
    sync::{mpsc, mpsc::TryRecvError, Arc},
    thread,
    time::Duration,
};

use common::{Network, UpdateMessage};
use dashmap::DashMap;
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token, Waker,
};
use signal_hook::iterator::Signals;
use slab::Slab;
use structopt::StructOpt;

mod database;
use database::{Database, Record};

const ACCEPTED_PROTO_VERSION: u8 = 66;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long, default_value = "./database", parse(from_os_str))]
    db_file: PathBuf,

    #[structopt(short, long, default_value = "0.0.0.0:1515")]
    address: SocketAddr,
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
    // event polling setup
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    // set up signal handling
    let mut signals = Signals::new(&[signal_hook::SIGINT])?;
    const SIGNAL: Token = Token(0);
    poll.registry()
        .register(&mut signals, SIGNAL, Interest::READABLE)?;
    // set up TCP
    let mut listener = TcpListener::bind(opts.address)?;
    const SERVER: Token = Token(1);
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;
    // set up Waker
    const WAKER: Token = Token(2);
    let waker = Arc::new(Waker::new(poll.registry(), WAKER)?);
    // create slab of threads
    let mut slab = Slab::new();
    // main event loop
    'main: loop {
        poll.poll(&mut events, None)?;
        for event in &events {
            match event.token() {
                SERVER => loop {
                    match listener.accept() {
                        Ok((conn, _addr)) => {
                            let clone = db.clone();
                            let w_clone = waker.clone();
                            let (tx, rx) = mpsc::channel::<()>();
                            let handle = thread::spawn(move || -> io::Result<()> {
                                handle(conn, clone)?;
                                w_clone.wake()?;
                                tx.send(()).expect("main hung up on child");
                                Ok(())
                            });
                            slab.insert((rx, handle));
                        }
                        Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => return Err(e),
                    }
                },
                SIGNAL => {
                    println!("Got SIGINT, dying politely...");
                    slab.shrink_to_fit();
                    for (_, handle) in slab.drain() {
                        // lol
                        handle.join().unwrap()?;
                    }
                    break 'main;
                }
                WAKER => {
                    // Since this goes over the entire slab, we don't need to add our own loop
                    slab.retain(|_, val| match val.0.try_recv() {
                        Ok(_) => false,
                        Err(e) if e == TryRecvError::Disconnected => false,
                        _ => true,
                    });
                }
                Token(_) => {}
            }
        }
    }
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
