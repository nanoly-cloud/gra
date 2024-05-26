use std::fmt::Debug;

use anyhow::{bail, Result};
use async_std::io;
use async_trait::async_trait;

use crate::storage::{DataStore, DataStoreError, DataType, MemoryStorage, Storage, Tier};

mod block;
pub use block::{Block, BLOCK_SIZE};

mod entry;
pub use entry::Entry;

mod peer;
pub use peer::Peer;

pub type Confidence = u64;

pub struct Models {
    blocks: Model<Block>,
    entries: Model<Entry>,
    // TODO: Add Peers, with fingerprint as key. Enables closest search
    // peers: Model<Fingerprint, Peer>,
}

impl Models {
    pub fn new(storage_tiers: Option<Vec<Tier>>) -> Result<Self> {
        let tiers = storage_tiers.unwrap_or(vec![Tier::Memory]);
        Ok(Self {
            blocks: Model::<Block>::new(&tiers)?,
            entries: Model::<Entry>::new(&tiers)?,
        })
    }
}

#[derive(Debug)]
pub struct Model<T: DataType> {
    stores: Vec<Storage<T>>,
}

impl<T: DataType> Model<T> {
    fn new(tiers: &Vec<Tier>) -> Result<Self> {
        if tiers.is_empty() {
            // TODO: Check POSIX, and return appropriate error
            bail!(DataStoreError::Invalid);
        }
        Ok(Self {
            stores: tiers.iter().cloned().map(|tier| tier.into()).collect(),
        })
    }
}

impl<T: DataType> Default for Model<T> {
    fn default() -> Self {
        Self {
            stores: Vec::from([Storage::Memory(MemoryStorage::new(4096))]),
        }
    }
}

#[async_trait]
impl<T: DataType> DataStore<T> for Model<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    async fn read(&self, key: &str) -> Result<T> {
        for store in &self.stores {
            if let Ok(data) = store.read(key).await {
                return Ok(data);
            }
        }
        bail!(DataStoreError::NotFound)
    }

    async fn write(&mut self, key: &str, data: &T) -> Result<()> {
        // let mut errors = vec![];
        // Should handle migration to lower store, if store is full, and eviction if necessary
        //   if let Err(err) = store.write(key, data).await {
        //       bail!(DataStoreError::EWRITE)
        //   }
        todo!()
    }

    async fn delete(&mut self, key: &str) -> Result<()> {
        // Is it worth migrating data up a tier?
        todo!()
    }

    async fn list(&self, key: Option<&str>) -> Result<Vec<&str>> {
        // Should zip all stores and return a list of keys
        todo!()
    }
}
