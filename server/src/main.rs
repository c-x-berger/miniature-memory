use std::{
    io,
    io::Read,
    net::{TcpListener, TcpStream},
};

use common::{Network, UpdateMessage};

const ACCEPTED_PROTO_VERSION: u8 = 66;

fn main() {
    println!("Hello, world!");
    println!("Binding to localhost:1515");
    mainloop().expect("Something has gone catastrophically wrong!");
}

fn invalid_data(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

fn handle(mut client: TcpStream) -> io::Result<()> {
    println!("handling new connection");
    let mut input_bytes: Vec<u8> = Vec::new();
    match client.read_to_end(&mut input_bytes) {
        Ok(n) => println!("read {} bytes", n),
        Err(e) => {
            println!("failed to read bytes");
            return Err(e);
        }
    }
    let rec_msg: UpdateMessage = match UpdateMessage::from_networking(&input_bytes) {
        Ok(v) => v,
        Err(_) => return Err(invalid_data("could not parse update message")),
    };
    println!("parsed message OK");
    if rec_msg.version() == ACCEPTED_PROTO_VERSION {
        println!("got good version version {}", ACCEPTED_PROTO_VERSION);
    }
    if rec_msg.correct_signature().unwrap() {
        println!(
            "got a good-signed message to update {} to {}",
            rec_msg.label(),
            rec_msg.value()
        );
    } else {
        println!("bad signature!");
        if rec_msg.key().is_some() {
            println!("  - key is present");
        }
        if rec_msg.signature().is_some() {
            println!("  - signature present");
        }
        println!("label: {}\nvalue: {}", rec_msg.label(), rec_msg.value());
    }
    return Ok(());
}

fn mainloop() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1515")?;
    for stream in listener.incoming() {
        handle(stream?)?;
    }
    Ok(())
}
