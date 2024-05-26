use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{DataStore, DataType, Storage};

mod disk;
use disk::DiskStorage;

mod memory;
pub use memory::MemoryStorage;

mod remote;
use remote::RemoteStorage;

#[derive(Debug, Clone)]
pub enum Tier {
    Memory,
    // Disk,
    // Remote,
}

impl Tier {
    pub fn new() -> Self {
        Tier::Memory
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Memory" => Tier::Memory,
            // "Disk" => Tier::Disk,
            // "Remote" => Tier::Remote,
            _ => Tier::Memory,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Tier::Memory => "Memory",
            // Tier::Disk => "Disk",
            // Tier::Remote => "Remote",
        }
    }
}

impl<T: DataType> Into<Storage<T>> for Tier {
    fn into(self) -> Storage<T> {
        match self {
            Tier::Memory => Storage::Memory(MemoryStorage::new(4096)),
            // Tier::Disk => Storage::Disk(DiskStorage::new(4096)),
            // Tier::Remote => Storage::Remote(RemoteStorage::new(4096)),
        }
    }
}

impl Default for Tier {
    fn default() -> Self {
        Tier::Memory
    }
}
