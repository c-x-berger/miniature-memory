use std::{convert::TryInto, io::Read};

use ed25519_dalek::{PublicKey, Signature, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};

use crate::update::UpdateMessage;

pub mod errors;
use errors::*;

/// The `Network` trait allows for objects to be serialized to and deserialized from bytes. These
/// bytes can then be sent over or recieved from the network.
///
/// Implementations should (if possible or reasonable) hold that
/// `Type::from_networking(obj.networking_bytes()) == obj`.
///
/// Be defensive in implementing [`from_networking()`], as it is likely to be taking input from the
/// outside world with little filtering. It's far better to error early than to be vulnerable to an
/// attack!
///
/// [`from_networking()`]: trait.Network.html#tymethod.from_networking
pub trait Network: Sized {
    /// Returns a representation of this object as a vector of bytes suitable to be sent over the
    /// network.
    fn networking_bytes(&self) -> Result<Vec<u8>, NetErr>;

    /// Attempt to reconstitute an instance of this object from bytes. This method may use the [`Read`]
    /// implementation on `bytes`, so it may be wise to first copy `bytes`.
    ///
    /// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
    fn from_networking(bytes: &[u8]) -> Result<Self, NetErr>;
}

impl Network for UpdateMessage {
    fn networking_bytes(&self) -> Result<Vec<u8>, NetErr> {
        if !self.is_signed() {
            return Err(NetErr::BytesNotReady);
        }
        let mut buf = Vec::<u8>::new();
        // when sending over network, big endian is standard for some reason
        buf.extend_from_slice(&u8::to_be_bytes(self.version()));
        // keys provide nice slice views but signatures -- which are larger -- don't
        buf.extend_from_slice(self.key()?.as_bytes());
        buf.extend_from_slice(&self.signature()?.to_bytes());
        // includes big-endian timestamp, label_s, label, value_s, value
        buf.extend_from_slice(&self.as_message());
        return Ok(buf);
    }

    fn from_networking(mut bytes: &[u8]) -> Result<Self, NetErr> {
        if bytes.len() < 107 {
            return Err(NetErr::NotEnoughData);
        }
        let mut version: [u8; 1] = [0];
        bytes.read_exact(&mut version)?;
        println!("read version {}", version[0]);
        let mut key_bytes: [u8; PUBLIC_KEY_LENGTH] = [0; PUBLIC_KEY_LENGTH];
        let mut sig_bytes: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];
        bytes.read_exact(&mut key_bytes)?;
        bytes.read_exact(&mut sig_bytes)?;
        let key = PublicKey::from_bytes(&key_bytes)?;
        let sig = Signature::from_bytes(&sig_bytes)?;
        println!("read key and signature OK");

        let mut timestamp: [u8; 8] = [0; 8];
        bytes.read_exact(&mut timestamp)?;
        let timestamp = u64::from_be_bytes(timestamp);
        println!("read timestamp of {}", timestamp);
        // alloc/declare buffers for label and size
        let mut label_s: [u8; 1] = [0];
        let mut value_s: [u8; 1] = [0];
        let mut label_v: Vec<u8>;
        let mut value_v: Vec<u8>;
        // read label/value
        // read size
        bytes.read_exact(&mut label_s)?;
        let size = u8::from_be_bytes(label_s).try_into().unwrap();
        // attempt to read `size` bytes to buffer
        // I'm fairly confident that a u8 fits into a usize
        label_v = vec![0; size];
        let read_size = bytes
            .read(label_v.as_mut_slice())
            .expect("could not read into label_v");
        if read_size < size {
            println!("Not enough data! {} < {}", read_size, size);
            println!("label_v has {} items, {:?}", label_v.len(), label_v);
            return Err(NetErr::NotEnoughData);
        }
        bytes.read_exact(&mut value_s)?;
        let size = u8::from_be_bytes(value_s).try_into().unwrap();
        value_v = vec![0; size];
        let read_size = bytes.read(&mut value_v)?;
        if read_size < size {
            return Err(NetErr::NotEnoughData);
        }

        let label: String = String::from_utf8(label_v)?;
        let value: String = String::from_utf8(value_v)?;

        let ret = UpdateMessage::new(
            u8::from_be_bytes(version),
            timestamp,
            label,
            value,
            Some(key),
            Some(sig),
        )?;
        // we have to return a box because something something memory safety
        Ok(ret)
    }
}
