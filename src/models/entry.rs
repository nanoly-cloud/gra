use ciborium::cbor;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::{
    hash::{CustomHash, Hash, HashOpts},
    models::Block,
    storage::{DataKey, DataType},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Entry(Hash, Hash);

impl Entry {
    pub fn new(hash: Hash, block: &Block) -> Self {
        let block_ref = match block {
            Block::Ref(hash) => hash.to_owned(),
            Block::Bytes(bytes) => Hash::from_bytes(&block.hash(), None),
            Block::Composite {
                timestamp,
                confidence,
                scope,
                data,
                children,
            } => Hash::from_bytes(&block.hash(), Some(HashOpts { key: scope.clone() })),
        };

        Self(hash, block_ref)
    }

    pub fn key(&self) -> &Hash {
        &self.0
    }

    pub fn value(&self) -> &Hash {
        &self.1
    }
}

impl DataKey for Entry {
    fn key(&self) -> String {
        self.0.to_hex()
    }
}

impl DataType for Entry {
    fn serialize(&self) -> Vec<u8> {
        let encoded = cbor!(self).expect("Failed to serialize Entry");
        encoded
            .into_bytes()
            .expect("Failed to convert Entry to bytes")
    }

    fn deserialize(bytes: &[u8]) -> Self {
        ciborium::de::from_reader(bytes).expect("Failed to deserialize Entry")
    }
}
