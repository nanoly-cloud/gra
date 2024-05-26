use async_trait::async_trait;
use std::{any, fmt::Debug};

use anyhow::Result;
use ciborium_io::{Read, Write};
use libp2p::{kad::store::MemoryStore, relay::client::new};

use crate::models::{Block, Entry};

mod datastore;
pub use datastore::{DataStore, DataStoreError};

mod tier;
pub use tier::{MemoryStorage, Tier};

pub trait DataKey {
    fn key(&self) -> String;
}

pub trait DataType: DataKey + Send + Sync + Clone + Debug {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum Storage<T: DataType> {
    // The Process should also be a storage tier, represented by a HashMap
    // Process(ProcessStorage),
    // This should become a file in /dev/shm
    Memory(MemoryStorage<T>),
    // Does a file in /tmp make any sense?
    // Disk(DiskStorage),
    // Disk(DiskStorage),
    // Remote(RemoteStorage),
}

#[async_trait]
impl<T: DataType> DataStore<T> for Storage<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    async fn read(&self, key: &str) -> Result<T> {
        match self {
            Storage::Memory(storage) => storage.read(key).await,
            // Storage::Disk(storage) => storage.read(key).await,
            // Storage::Remote(storage) => storage.read(key).await,
        }
    }

    async fn write(&mut self, key: &str, value: &T) -> Result<()> {
        match self {
            Storage::Memory(storage) => storage.write(key, value).await,
            // Storage::Disk(storage) => storage.insert(key, value),
            // Storage::Remote(storage) => storage.insert(key, value),
        }
    }

    async fn delete(&mut self, key: &str) -> Result<()> {
        match self {
            Storage::Memory(storage) => storage.delete(key).await,
            // Storage::Disk(storage) => storage.delete(key),
            // Storage::Remote(storage) => storage.delete(key),
        }
    }

    async fn list(&self, key: Option<&str>) -> Result<Vec<&str>> {
        match self {
            Storage::Memory(storage) => storage.list(key).await,
            // Storage::Disk(storage) => storage.list(),
            // Storage::Remote(storage) => storage.list(),
        }
    }
}
