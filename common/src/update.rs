use std::{convert::TryInto, option::NoneError};

use ed25519_dalek::{Keypair, PublicKey, Signature};

/// Update message structure. Layout from http://wiki.ucis.nl/MARC
pub struct UpdateMessage {
    version: u8,
    public_key: Option<PublicKey>,
    signature: Option<Signature>,
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
        key: Option<PublicKey>,
        signature: Option<Signature>,
    ) -> Result<Self, ()> {
        if label.len() > 256 || value.len() > 256 {
            return Err(());
        }
        Ok(Self {
            version,
            timestamp,
            label,
            value,
            public_key: key,
            signature,
        })
    }

    /// Returns this message as it should have been serialized prior to signing.
    pub fn as_message(&self) -> Vec<u8> {
        let mut to_sign: Vec<u8> = self.timestamp.to_be_bytes().to_vec();
        for val in &[self.label(), self.value()] {
            let len: u8 = val.len().try_into().unwrap();
            to_sign.extend_from_slice(&len.to_be_bytes());
            to_sign.extend(val.bytes());
        }
        return to_sign;
    }

    /// Sets this message's public key and signs it with the provided keypair.
    pub fn sign(&mut self, key: &Keypair) {
        let pub_key = key.public;
        self.public_key = Some(pub_key);
        let to_sign = &self.as_message();
        let sign = key.sign(to_sign);
        assert!(key.verify(to_sign, &sign).is_ok());
        self.signature = Some(sign);
    }

    /// Checks if this update is validly signed.
    ///
    /// - If: `self.key().is_some()` and `self.signature().is_some()`:
    ///   - If `signature` is valid for `self.as_message()` and `key`:
    ///     - Return `Ok(true)`
    ///   - Else: Return `Ok(false)`
    /// - Else: return NoneError
    pub fn correct_signature(&self) -> Result<bool, NoneError> {
        match self.signature() {
            Some(signature) => {
                return match self.key() {
                    Some(key) => Ok(key.verify(&self.as_message(), &signature).is_ok()),
                    None => Err(NoneError),
                }
            }
            None => Err(NoneError),
        }
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn key(&self) -> Option<PublicKey> {
        self.public_key
    }

    pub fn signature(&self) -> Option<Signature> {
        self.signature
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

    /// Return `true` if a signature is **present.** To check correctness, see [`correct_signature`].
    ///
    /// [`correct_signature`]: struct.UpdateMessage.html#method.correct_signature
    pub fn is_signed(&self) -> bool {
        return self.signature.is_some();
    }
}
