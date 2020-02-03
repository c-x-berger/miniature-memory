use std::{
    convert::TryInto,
    io,
    io::{IoSliceMut, Read},
    net::{TcpListener, TcpStream},
    time::{SystemTime, UNIX_EPOCH},
};

use ed25519_dalek::{Keypair, PublicKey, Signature, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use rand::rngs::OsRng;

use common::UpdateMessage;

const ACCEPTED_PROTO_VERSION: u8 = 6;

fn main() {
    println!("Hello, world!");

    let mut csprng = OsRng {};
    let keypair = Keypair::generate(&mut csprng);

    let start = SystemTime::now();
    let nix_time = start.duration_since(UNIX_EPOCH).expect("fuck timezones");

    println!("testing update message struct");
    let update = UpdateMessage::new(
        66,
        nix_time.as_secs(),
        "jimmyhendrix.ano".to_string(),
        "why.am.i.like.this".to_string(),
        None,
        None,
    );
    match update {
        Ok(mut v) => {
            v.sign(&keypair).expect("signing error");
            println!("signed ok?")
        }
        Err(_) => println!("create error"),
    }
}

fn invalid_data(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

fn handle(mut client: TcpStream) -> io::Result<()> {
    // TODO: Multithread this
    let mut version: [u8; 1] = [0; 1];
    client.read_exact(&mut version)?;
    if version[0] != ACCEPTED_PROTO_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid protocol version",
        ));
    }
    let mut publik: [u8; PUBLIC_KEY_LENGTH] = [0; PUBLIC_KEY_LENGTH];
    let mut signat: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];
    client.read_vectored(&mut [IoSliceMut::new(&mut publik), IoSliceMut::new(&mut signat)])?;
    let key = match PublicKey::from_bytes(&publik) {
        Ok(key) => key,
        Err(_) => return Err(invalid_data("bad public key")),
    };
    let signature = match Signature::from_bytes(&signat) {
        Ok(sig) => sig,
        Err(_) => return Err(invalid_data("could not read sig")),
    };
    // read in message
    let mut timestmp: [u8; 8] = [0; 8];
    let mut label_s: [u8; 1] = [0];
    client.read_vectored(&mut [
        IoSliceMut::new(&mut timestmp),
        IoSliceMut::new(&mut label_s),
    ])?;
    let mut message: Vec<u8> = Vec::new();
    message.extend_from_slice(&timestmp);
    message.extend_from_slice(&label_s);
    let timestamp = u64::from_be_bytes(timestmp);
    // if this panics, submit proof to me to claim your prize
    let label_size: usize = u8::from_be_bytes(label_s).try_into().unwrap();
    // make space for label
    let mut label: Vec<u8> = Vec::with_capacity(label_size);
    let actual_label_bytes = client.read(label.as_mut())?;
    if actual_label_bytes < label_size {
        // something has gone horribly wrong and the client should now sod off
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected more bytes than recieved",
        ));
    }
    message.extend_from_slice(&label);
    // reading value is similar to label, but we need value_s first
    let mut value_s: [u8; 1] = [0];
    client.read_exact(&mut value_s)?;
    message.extend_from_slice(&value_s);
    let value_size: usize = u8::from_be_bytes(value_s).try_into().unwrap();
    let mut value: Vec<u8> = Vec::with_capacity(value_size);
    let actual_values_bytes = client.read(&mut value)?;
    if actual_values_bytes < value_size {
        // something has gone horribly wrong and the client should now sod off
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected more bytes than recieved",
        ));
    }
    message.extend_from_slice(&value);
    // check sig valid
    Ok(())
}

fn mainloop() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1515")?;
    for stream in listener.incoming() {
        handle(stream?)?;
    }
    Ok(())
}
