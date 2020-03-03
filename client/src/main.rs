use std::{
    fs::File,
    io,
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use structopt::StructOpt;

use common::{Network, UpdateMessage};

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(short, long, help = "Server to connect to")]
    server: String,
    #[structopt(short, long, help = "Port to connect to", default_value = "1515")]
    port: u16,
    #[structopt(
        short,
        long,
        help = "File containing key",
        default_value = "./ed_marc.key"
    )]
    keypath: PathBuf,
}

fn main() -> Result<(), io::Error> {
    let opt = Options::from_args();
    let mut csprng = OsRng {};

    // ref avoids move
    let keyfile = File::open(&opt.keypath);

    let keypair: Keypair = match keyfile {
        Ok(f) => {
            let reader = io::BufReader::new(f);
            Keypair::from_bytes(&reader.bytes().map(|b| b.unwrap()).collect::<Vec<_>>())
                .expect("invalid key")
        }
        Err(e) => {
            let key = Keypair::generate(&mut csprng);
            if e.kind() == io::ErrorKind::NotFound {
                println!("writing new key");
                let out = File::create(&opt.keypath)?;
                let mut writer = io::BufWriter::new(out);
                writer.write(&key.to_bytes())?;
            }
            key
        }
    };
    println!("loaded key");

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

    // we'll just assume they gave a real IP:port set
    let mut stream =
        TcpStream::connect((opt.server.as_str(), opt.port)).expect("failed to connect to server!");
    println!(
        "connected to {}:{}, trying to send message...",
        opt.server, opt.port
    );
    stream.write(&update.networking_bytes()?)?;

    Ok(())
}
