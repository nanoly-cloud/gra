use anyhow::Result;
use hex;
use std::{
    borrow::Borrow,
    fs,
    ops::Deref,
    path::PathBuf,
    time::{Duration, Instant},
};

use blake3::{Hasher, OUT_LEN};
use chrono::Utc;
use ciborium::{cbor, from_reader, into_writer};
use ciborium_io::{Read, Write};
use libp2p::{kad::Record, PeerId};
use serde::de::IntoDeserializer;
use zerocopy::AsBytes;

use crate::{
    hash::{CustomHash, Hash, HashOpts},
    reader::add_path,
    storage::{DataKey, DataType},
};

type Confidence = usize;

pub const BLOCK_SIZE: usize = 32;

/// Terminology:
/// - Block: A combination of words.
/// - Chunk: A number of bytes, equal to BLOCK_SIZE.
/// - Element: A block or a hash.

#[repr(C)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, PartialOrd)]
#[serde(untagged)]
pub enum Block {
    Bytes(Vec<[u8; BLOCK_SIZE]>),
    Ref(Hash),
    Composite {
        timestamp: chrono::DateTime<chrono::Utc>,
        // TODO: Remove PeerId
        confidence: Confidence,
        scope: Option<Hash>,
        // CONSIDER: Change to something like below
        // data: Option<Vec<Box<Block>>>,
        data: Option<Box<Block>>,
        children: Option<Vec<Box<Block>>>,
    },
}

impl Block {
    pub fn new(
        timestamp: chrono::DateTime<chrono::Utc>,
        peer_id: PeerId,
        confidence: Confidence,
        scope: Option<Hash>,
        data: Vec<u8>,
        children: Option<Vec<Box<Block>>>,
    ) -> Self {
        let mut chunks: Vec<[u8; BLOCK_SIZE]> = data
            .chunks_exact(BLOCK_SIZE)
            .map(|chunk| {
                <[u8; BLOCK_SIZE]>::try_from(chunk)
                    .expect(&format!("Chunk length is guaranteed to be {BLOCK_SIZE}"))
            })
            .collect();

        let remaining = data.chunks_exact(BLOCK_SIZE).remainder();
        if !remaining.is_empty() {
            let chunk_len = remaining.len();
            let mut partial_chunk = [0; BLOCK_SIZE];
            partial_chunk[..chunk_len].copy_from_slice(&remaining[..chunk_len]);
            chunks.push(partial_chunk);
        };

        Block::Composite {
            timestamp,
            confidence,
            scope,
            data: Some(Box::new(Block::Bytes(chunks))),
            children, //children.into_iter().map(|block| Box::new(block)).collect(),
        }
    }

    // Serialize a block into CBOR
    pub fn to_cbor(&self) -> Result<Vec<u8>> {
        // Create a buffer to hold the CBOR output
        let mut buffer = Vec::new();

        // Serialize the block into the buffer
        into_writer(self, &mut buffer)?;

        Ok(buffer)
    }

    // Deserialize a block from CBOR
    pub fn from_cbor(cbor_data: &[u8]) -> Result<Self> {
        let block: Block = from_reader(cbor_data)?;
        Ok(block)
    }
}

impl Into<Hash> for Block {
    fn into(self) -> Hash {
        match &self {
            Block::Ref(hash) => hash.clone(),
            Block::Bytes(bytes) => {
                let flattened: Vec<u8> = bytes.iter().flat_map(|hash| hash.to_owned()).collect();
                Hash::new(flattened.as_slice(), None)
            }
            Block::Composite {
                timestamp,
                confidence,
                scope,
                data,
                children,
            } => Hash::new(
                self.borrow()
                    .to_cbor()
                    .expect("Failed to serialize to CBOR")
                    .as_bytes(),
                Some(HashOpts { key: scope.clone() }),
            ),
        }
    }
}

impl Into<Vec<u8>> for Block {
    fn into(self) -> Vec<u8> {
        let mut encoded = Vec::new();
        into_writer(&self, &mut encoded);
        encoded
    }
}

impl DataKey for Block {
    fn key(&self) -> String {
        String::from_utf8_lossy(&self.hash()).to_string()
    }
}

impl CustomHash for Block {
    fn hash(&self) -> [u8; OUT_LEN] {
        let mut hasher = match self {
            Block::Composite { scope, .. } => match scope {
                Some(scope) => blake3::Hasher::new_keyed(scope.as_bytes()),
                None => blake3::Hasher::new(),
            },
            _ => blake3::Hasher::new(),
        };

        match self {
            Block::Ref(hash) => {
                hasher.update(hash.as_bytes());
            }
            Block::Bytes(bytes) => bytes.iter().for_each(|hash| {
                hasher.update(hash.as_bytes());
            }),
            Block::Composite {
                timestamp,
                confidence,
                scope,
                data,
                children,
            } => {
                hasher.update(timestamp.to_utc().to_string().as_bytes());
                hasher.update(confidence.as_bytes());
                if let Some(data) = data {
                    hasher.update(
                        &data
                            .to_cbor()
                            .expect("Failed to serialize Block")
                            .as_bytes(),
                    );
                } else {
                    hasher.update(&[]);
                };

                children.iter().for_each(|element| {
                    hasher.update(
                        element
                            .iter()
                            .flat_map(|el| {
                                el.to_owned().to_cbor().expect("Failed to serialize Block")
                            })
                            .collect::<Vec<u8>>()
                            .as_slice(),
                    );
                });
            }
        }

        *hasher.finalize().as_bytes()
    }
}

impl DataType for Block {
    fn serialize(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        into_writer(self, &mut encoded).expect("Failed to serialize Block");
        encoded
    }

    fn deserialize(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        from_reader(data).expect("Failed to decode Block")
    }
}

// type Payload = Element;

// #[repr(C)]
// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, PartialOrd)]
// pub struct Block(
//     chrono::DateTime<chrono::Utc>,
//     PeerId,
//     Confidence,
//     Option<[u8; 32]>,
//     Box<Element>,
//     Vec<Element>,
// );

// impl Block {
//     pub fn new(
//         peer_id: PeerId,
//         confidence: Confidence,
//         scope: Option<&str>,
//         data: Element,
//         children: Vec<Element>,
//     ) -> Self {
//         let key: Option<[u8; 32]> = if let Some(scope) = scope {
//             Some(blake3::new_keyed(scope))
//         } else {
//             None
//         };
//         Block(
//             Utc::now(),
//             peer_id,
//             confidence,
//             key,
//             Box::new(data),
//             children,
//         )
//     }

//     pub fn hash(&self) -> Hash {
//         self.to_owned().into()
//     }

//     pub fn to_hash(&self) -> Hash {
//         self.hash()
//     }

//     // pub fn to_vec(&self) -> Vec<u8> {
//     //     *self
//     //         .4
//     //         .as_ref()
//     //         .to_vec()
//     //         .map(|element| match element.as_ref() {
//     //             Element::Block(block) => block.to_vec(),
//     //             Element::Bytes(hash) => hash.as_bytes().to_vec(),
//     //         })
//     //         .flatten()
//     //         .collect()
//     // }
// }

// impl Into<Hash> for Block {
//     fn into(self) -> Hash {
//         let mut hasher = match self.3 {
//             Some(scope) => blake3::Hasher::new_keyed(&scope),
//             None => blake3::Hasher::new(),
//         };
//         hasher.update(self.0.to_utc().to_string().as_bytes());
//         hasher.update(&self.1.to_bytes());
//         hasher.update(self.2.as_bytes());
//         self.3.iter().for_each(|element| {
//             hasher.update(element.as_bytes());
//         });
//         let hash = hasher.finalize();
//         Hash(hash, None)
//     }
// }

// impl Into<Vec<u8>> for Block {
//     fn into(self) -> Vec<u8> {
//         unimplemented!()
//         // self.4
//         //     .iter()
//         //     .map(|element| element.hash().as_bytes().to_vec())
//         //     .flatten()
//         //     .collect::<Vec<u8>>()
//     }
// }

impl Into<Record> for Block {
    fn into(self) -> Record {
        match self {
            Block::Ref(hash) => Record {
                key: hash.into(),
                value: Vec::new(),
                publisher: None,
                expires: Some(Instant::now() + Duration::from_secs(3600)),
            },
            Block::Bytes(bytes) => Record {
                key: bytes
                    .iter()
                    .flat_map(|hash| hash.to_owned())
                    .collect::<Vec<u8>>()
                    .into(),
                value: Vec::new(),
                publisher: None,
                expires: None,
            },
            Block::Composite {
                timestamp,
                confidence,
                ref scope,
                ref data,
                ref children,
            } => {
                let hash: Hash = self.clone().into();
                Record {
                    key: hash.into(),
                    value: self.to_cbor().expect("Failed to serialize Block"),
                    publisher: Some(PeerId::random()),
                    expires: None,
                }
            }
        }
        // let key = match self.3 {
        //     Some(k) => {
        //         let key = String::from_utf8(k.to_vec()).unwrap_or("".to_string());
        //         format!("{:?}|{:?}", key, self.hash().to_hex())
        //     }
        //     None => format!("{:?}", self.hash()),
        // };
        // Record {
        //     key: RecordKey::new(&key),
        //     value: self.to_vec(),
        //     publisher: Some(self.1),
        //     // TODO: Add time
        //     expires: Some(Instant::now()),
        // }
    }
}

impl From<&[u8]> for Block {
    fn from(value: &[u8]) -> Self {
        from_reader(value).expect("Failed to decode Block")
    }
}

// impl From<Record> for Block {
//     fn into(self) -> Record {
//         let key: String = self
//             .3
//             .clone()
//             .map(|hash| hash.to_hex())
//             .unwrap_or_else(|| "".to_string())
//             .into();
//         let hash: Hash = &self.into();
//         Record {
//             key: RecordKey::from(format!("{:?}|{:?}", key, hash)),
//             value: self.to_vec(),
//             publisher: Some(self.1),
//             // TODO: Add time
//             expires: Some(Instant::now()),
//         }
//     }
// }

// #[derive(Debug, Default, Clone)]
// pub struct Blocks(Arc<Mutex<HashMap<Hash, Block>>>);

// impl DataStore for Blocks {
//     // pub fn new() -> Self {
//     //     Blocks(Arc::new(Mutex::new(HashMap::new())))
//     // }

//     pub fn get(self, hash: &Hash) -> Result<Option<Block>> {
//         let block = self.0.lock().unwrap().get(hash).clone().cloned();
//         Ok(block)
//     }

//     pub fn insert(&mut self, hash: &Hash, block: &Block) -> Result<Option<Block>> {
//         let block = self
//             .0
//             .lock()
//             .unwrap()
//             .insert(hash.to_owned(), block.to_owned());

//         Ok(block)
//     }

//     pub fn remove(&mut self, hash: &Hash) -> Result<Option<Block>> {
//         let block = self.0.lock().unwrap().remove(hash);

//         Ok(block)
//     }
// }
