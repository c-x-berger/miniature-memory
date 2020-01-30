use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

use common::UpdateMessage;

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
        "jimmyhendrix.ano",
        "why.am.i.like.this",
    );
    match update {
        Ok(mut v) => {
            v.sign(&keypair).expect("signing error");
            println!("signed ok?")
        }
        Err(_) => println!("create error"),
    }
}
