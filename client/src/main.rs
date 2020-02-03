use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

use common::UpdateMessage;

fn main() {
    println!("creating key");

    let mut csprng = OsRng {};
    let keypair = Keypair::generate(&mut csprng);

    let start = SystemTime::now();
    let nix_time = start.duration_since(UNIX_EPOCH).expect("fuck timezones");

    let mut update = UpdateMessage::new(
        66,
        nix_time.as_secs(),
        "test.spoon".to_string(),
        "localhost".to_string(),
        None,
        None,
    )
    .unwrap();
    update
        .sign(&keypair)
        .expect("could not generate message signature");
}
