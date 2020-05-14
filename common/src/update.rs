use std::convert::TryInto;

use ed25519_dalek::{Keypair, PublicKey, Signature, SignatureError};

/// Update message structure. Layout from http://wiki.ucis.nl/MARC
#[derive(Debug, PartialEq, Eq)]
pub struct UpdateMessage {
    version: u8,
    key: PublicKey,
    signature: Signature,
    /// We will store this number as the target's native endianness, and to_be it in networking
    timestamp: u64,
    label: String,
    value: String,
}

impl UpdateMessage {
    pub fn new(
        version: u8,
        timestamp: u64,
        label: String,
        value: String,
        key: PublicKey,
        signature: Signature,
    ) -> Result<Self, ()> {
        if label.len() > 256 {
            return Err(());
        }
        Ok(Self {
            version,
            timestamp,
            label,
            value,
            key,
            signature,
        })
    }

    pub fn bytes_to_sign(timestamp: u64, label: &str, value: &str) -> Vec<u8> {
        let mut bytes: Vec<u8> = timestamp.to_be_bytes().to_vec();
        let len: u8 = label.len().try_into().unwrap();
        bytes.push(len.to_be());
        bytes.extend(label.bytes());
        bytes.extend(value.bytes());
        bytes
    }

    /// Returns this message as it should have been serialized prior to signing.
    pub fn as_message(&self) -> Vec<u8> {
        Self::bytes_to_sign(self.timestamp, self.label(), self.value())
    }

    /// Sets this message's public key and signs it with the provided keypair.
    pub fn sign(&mut self, key: &Keypair) {
        let to_sign = &self.as_message();
        let sign = key.sign(to_sign);
        assert!(key.verify(to_sign, &sign).is_ok());
        self.signature = sign;
        self.key = key.public;
    }

    /// Checks if this update is validly signed.
    ///
    /// - If `signature` is valid for `self.as_message()` and `key`:
    ///   - Return `Ok(true)`
    /// - Else: Return `Ok(false)`
    pub fn correct_signature(&self) -> Result<(), SignatureError> {
        self.key().verify(&self.as_message(), &self.signature)
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn key(&self) -> &PublicKey {
        &self.key
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
