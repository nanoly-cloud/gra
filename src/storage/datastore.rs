use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use ciborium_io::{Read, Write};
use derive_more::{Display, From};
use thiserror::Error;

use crate::hash::Hash;
use crate::models::Block;
use crate::storage::DataType;

use libc;

/// The `DataStore` trait defines the interface for a data store.
#[async_trait]
pub trait DataStore<T: DataType + Send + Sync + Debug> {
    /// Returns the block associated with the key. If the key is not found, returns None
    async fn read(&self, key: &str) -> Result<T>;

    /// Inserts a block into the data store.
    async fn write(&mut self, key: &str, value: &T) -> Result<()>;

    /// Removes the block for the given key from the data store
    async fn delete(&mut self, key: &str) -> Result<()> {
        todo!()
    }

    /// Returns a list of all the keys in the data store
    async fn list(&self, key: Option<&str>) -> Result<Vec<&str>> {
        todo!()
    }

    /// Creates a filter of the data store, for a compact, and communicable proof of membership
    async fn filter(&self) -> Result<&str> {
        todo!()
    }
    /// Returns true if the key is in the data store.
    async fn contains(&self, key: &str) -> Result<bool> {
        todo!()
    }

    /// Returns the number of blocks in the data store
    async fn len(&self) -> Result<usize> {
        todo!()
    }

    /// Returns the size of the data store in bytes
    async fn size(&self) -> Result<u64> {
        todo!()
    }

    /// Register a callback function that will be called when the data store changes.
    async fn on_change(&self, func: fn() -> Result<()>) -> Result<()> {
        todo!()
    }

    /// Prevent storage, and transmission of a block
    async fn forbid(&self, key: &str) -> Result<()> {
        todo!()
    }

    /// Re-enable storage, and transmission of a block
    async fn allow(&self, key: &str) -> Result<()> {
        todo!()
    }

    // XXX: A bit more dangeroos for now.
    // ///
    // async fn clear(&mut self) -> Result<()> {
    //     todo!()
    // }
}

/// The `DataStoreError` enum represents the possible errors that can occur when interacting with a data store.
/// The error variants are mapped to the underlying POSIX error codes.
#[derive(Debug, Display, Error)]
pub enum DataStoreError {
    AccessDenied,
    PermissionDenied,
    NotFound,
    ConnectionRefused,
    Invalid,
    Unknown(i32),
}

impl DataStoreError {
    pub fn from_errno(errno: i32) -> DataStoreError {
        match errno {
            libc::EACCES => DataStoreError::AccessDenied,
            libc::ENOENT => DataStoreError::NotFound,
            libc::ECONNREFUSED => DataStoreError::ConnectionRefused,
            libc::EINVAL => DataStoreError::Invalid,
            libc::EPERM => DataStoreError::PermissionDenied,
            _ => DataStoreError::Unknown(errno),
        }
    }
}
