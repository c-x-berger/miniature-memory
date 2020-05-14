use std::convert::TryInto;

use common::UpdateMessage;
use dashmap::DashMap;
use ed25519_dalek::{PublicKey, Signature};
use serde::{Deserialize, Serialize};

type Map = DashMap<String, Record>;

pub struct Database {
    records: Map,
}

impl Database {
    pub fn new(records: Map) -> Self {
        Database { records }
    }

    pub fn records(&self) -> &Map {
        &self.records
    }

    /// Returns true if `new_rec` can be inserted into the database at `key` without stealing
    /// someone's claim, i.e., `key` is unclaimed or is claimed by the same key as `new_rec`.
    fn can_insert_at(&self, key: &str, new_rec: &Record) -> bool {
        if self.records().contains_key(key) {
            // compare keys
            let current = self.records().get(key).unwrap();
            current.value().owner == new_rec.owner
        } else {
            true
        }
    }

    pub fn add_record(&self, key: &str, new: Record) -> bool {
        if self.can_insert_at(key, &new) && new.check_signature(key).is_ok() {
            self.records().insert(String::from(key), new);
            return true;
        }
        false
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    timestamp: u64,
    value: String,
    owner: PublicKey,
    signature: Signature,
}

impl Record {
    pub fn new(timestamp: u64, value: String, owner: PublicKey, signature: Signature) -> Self {
        Record {
            timestamp,
            value,
            owner,
            signature,
        }
    }

    fn check_signature(&self, label: &str) -> Result<(), ()> {
        if label.len() > 256 {
            Err(())
        } else {
            // construct array
            let mut bytes: Vec<u8> = self.timestamp.to_be_bytes().to_vec();
            let label_size: u8 = label.len().try_into().unwrap();
            bytes.push(label_size.to_be());
            bytes.extend(label.bytes());
            bytes.extend(self.value.bytes());
            match self.owner.verify(&bytes, &self.signature) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }
    }
}

impl From<UpdateMessage> for Record {
    fn from(message: UpdateMessage) -> Self {
        Self::new(
            message.timestamp(),
            String::from(message.value()),
            *message.key(),
            *message.signature(),
        )
    }
}
