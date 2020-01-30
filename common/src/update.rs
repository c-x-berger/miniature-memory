use std::{convert::TryInto, num::TryFromIntError};

use ed25519_dalek::Keypair;

/// Update message structure. Layout from http://wiki.ucis.nl/MARC
pub struct UpdateMessage<'a> {
    version: u8,
    public_key: Option<[u8; 32]>,
    signature: Option<[u8; 64]>,
    /// We will store this number as the target's native endianness, and to_be it in networking
    timestamp: u64,
    label: &'a str,
    value: &'a str,
}

impl<'a> UpdateMessage<'a> {
    pub fn new(version: u8, timestamp: u64, label: &'a str, value: &'a str) -> Result<Self, ()> {
        if label.len() > 256 || value.len() > 256 {
            return Err(());
        }
        Ok(Self {
            version,
            timestamp,
            label,
            value,
            public_key: None,
            signature: None,
        })
    }

    pub fn sign(&mut self, key: &Keypair) -> Result<(), TryFromIntError> {
        let pub_key = key.public;
        self.public_key = Some(pub_key.to_bytes());
        let mut to_sign: Vec<u8> = self.timestamp.to_be_bytes().to_vec();
        for val in &[self.label, self.value] {
            to_sign.push(val.len().try_into()?);
            to_sign.extend(val.bytes());
        }
        let sign = key.sign(to_sign.as_slice());
        assert!(key.verify(to_sign.as_slice(), &sign).is_ok());
        self.signature = Some(sign.to_bytes());
        Ok(())
    }
}
