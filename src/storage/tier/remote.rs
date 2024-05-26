use anyhow::Result;
use async_trait::async_trait;
use ciborium_io::{Read, Write};
use hashbrown::HashMap;

use crate::node::Node;

use super::{DataStore, DataType};

pub struct RemoteStorage {
    // node: Node,
}

impl RemoteStorage {
    pub fn new() -> Self {
        RemoteStorage {}
    }
}

#[async_trait]
impl<T: DataType> DataStore<T> for RemoteStorage {
    async fn read(&self, key: &str) -> Result<T> {
        todo!()
    }

    async fn write(&mut self, key: &str, value: &T) -> Result<()> {
        todo!()
    }

    async fn delete(&mut self, key: &str) -> Result<()> {
        todo!()
    }

    async fn list(&self, key: Option<&str>) -> Result<Vec<&str>> {
        todo!()
    }
}
