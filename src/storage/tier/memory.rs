use anyhow::Result;
use async_std::sync::RwLock;
use async_trait::async_trait;
use ciborium_io::{Read, Write};
use hashbrown::HashMap;

use crate::storage::{DataStoreError, DataType};

use super::DataStore;

#[derive(Debug)]
pub struct MemoryStorage<T: DataType> {
    data: RwLock<HashMap<String, T>>,
    max_size: usize,
}

impl<T: DataType> MemoryStorage<T> {
    pub fn new(max_size: usize) -> Self {
        MemoryStorage {
            data: RwLock::new(HashMap::new()),
            max_size: 0,
        }
    }
}

#[async_trait]
impl<T: DataType> DataStore<T> for MemoryStorage<T> {
    async fn write(&mut self, key: &str, value: &T) -> Result<()> {
        todo!()
        // self.data.insert(key, value);
        // Ok(())
    }

    async fn read(&self, key: &str) -> Result<T> {
        todo!()
        // self.data
        //     .read()
        //     .await
        //     .get(key)
        //     .cloned()
        //     .ok_or(DataStoreError::NotFound)
    }

    async fn delete(&mut self, key: &str) -> Result<()> {
        todo!()
        // self.data.remove(key);
        // Ok(())
    }

    async fn list(&self, key: Option<&str>) -> Result<Vec<&str>> {
        todo!()
        // self.data.values().collect()
    }
}
