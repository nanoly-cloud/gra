use anyhow::{anyhow, Result};
use async_std::path::PathBuf;
use async_trait::async_trait;
use ciborium::{from_reader, into_writer};
use ciborium_io::{Read, Write};
use hashbrown::HashMap;
use std::fs::{File, OpenOptions};
use tracing::debug;

use super::{DataStore, DataType};

/// A disk-based key-value store
pub struct DiskStorage {
    root_dir: PathBuf,
    max_size: usize,
}

impl DiskStorage {
    pub fn new(root_dir: PathBuf, max_size: usize) -> Self {
        DiskStorage { root_dir, max_size }
    }

    fn get_file_path(&self, key: &str) -> PathBuf {
        self.root_dir.join(key)
    }
}

#[async_trait]
impl<T: DataType> DataStore<T> for DiskStorage
where
    T: serde::Serialize + serde::de::DeserializeOwned + Write + Read + Send + Sync + Clone,
{
    async fn write(&mut self, key: &str, value: &T) -> Result<()> {
        let file_path = self.get_file_path(&key.to_string());
        debug!("Writing to file: {:?}", file_path);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)?;
        todo!()
        // into_writer(&file, value.to_owned()).map_err(|e| anyhow!(e))
    }

    async fn read(&self, key: &str) -> Result<T> {
        let file_path = self.get_file_path(&key.to_string());
        let file = File::open(file_path)?;
        // from_reader(file).map_err(|e| e.into());
        todo!()
    }

    async fn delete(&mut self, key: &str) -> Result<()> {
        let file_path = self.get_file_path(&key.to_string());
        async_std::fs::remove_file(file_path).await?;
        todo!()
    }

    async fn list(&self, key: Option<&str>) -> Result<Vec<&str>> {
        todo!()
    }
}
