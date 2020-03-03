use std::{error::Error, fmt, io, option::NoneError, string::FromUtf8Error};

use ed25519_dalek::SignatureError;

/// Errors that might occur in dealing with objects over the network.
#[derive(Debug)]
pub enum NetErr {
    BytesNotReady,
    NotEnoughData,
    MalformedData,
}

impl fmt::Display for NetErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Networking Error! (make this more descriptive, please!)")
    }
}

impl Error for NetErr {}

impl From<()> for NetErr {
    fn from(_: ()) -> Self {
        NetErr::MalformedData
    }
}

impl From<NoneError> for NetErr {
    fn from(_: NoneError) -> Self {
        NetErr::MalformedData
    }
}

impl From<SignatureError> for NetErr {
    fn from(_: SignatureError) -> Self {
        NetErr::MalformedData
    }
}

impl From<FromUtf8Error> for NetErr {
    fn from(_: FromUtf8Error) -> Self {
        NetErr::MalformedData
    }
}

impl From<io::Error> for NetErr {
    fn from(error: io::Error) -> Self {
        return match error.kind() {
            io::ErrorKind::UnexpectedEof => NetErr::NotEnoughData,
            _ => NetErr::MalformedData,
        };
    }
}

impl From<NetErr> for io::Error {
    fn from(e: NetErr) -> Self {
        return io::Error::new(
            io::ErrorKind::Other,
            format!("NetErr raised from common: {:?}", e),
        );
    }
}
