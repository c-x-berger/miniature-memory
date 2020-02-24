use std::{
    io,
    net::TcpStream,
    time::{SystemTime, UNIX_EPOCH},
};

use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use std::io::Write;

use common::{Network, UpdateMessage};

fn main() -> Result<(), io::Error> {
    println!("creating key");
    let mut csprng = OsRng {};
    let keypair = Keypair::generate(&mut csprng);
    println!("created key");

    let start = SystemTime::now();
    let nix_time = start.duration_since(UNIX_EPOCH).expect("fuck timezones");

    println!("creating and signing update...");
    let mut update = UpdateMessage::new(
        66,
        nix_time.as_secs(),
        "test.spoon".to_string(),
        "localhost".to_string(),
        None,
        None,
    )
    .unwrap();
    update.sign(&keypair);

    // ask for input
    let dest_server = loop {
        print!("connect to: ");
        io::stdout().flush().unwrap();
        let mut tmp = String::new();
        io::stdin().read_line(&mut tmp)?;
        match tmp.trim() {
            "" => continue,
            _ => break tmp,
        }
    };
    let dest_server = dest_server.trim();
    // we'll just assume they gave a real IP:port set
    let mut stream = TcpStream::connect(&dest_server).expect("failed to connect to server!");
    println!("connected to {}, trying to send message...", dest_server);
    stream.write(&update.networking_bytes()?)?;

    Ok(())
}
