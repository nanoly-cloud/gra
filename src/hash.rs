use std::fmt::{self, Debug, Display, Formatter};

use blake3::{Hash as B3Hash, OUT_LEN};
use ciborium::{cbor, into_writer};
use libp2p::{kad::RecordKey, request_response::cbor};
use tracing::info;
use zerocopy::AsBytes;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct HashOpts {
    pub key: Option<Hash>,
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Hash(B3Hash, Option<Box<Hash>>);

/// Copies the bytes of the key into a new vector.

impl Hash {
    pub fn new(data: &[u8], opts: Option<HashOpts>) -> Self {
        if let Some(opts) = opts {
            if let Some(key) = opts.key {
                return Hash(
                    blake3::keyed_hash(&key.as_bytes(), data),
                    Some(Box::new(key)),
                );
            }
        }
        Hash(blake3::hash(data).into(), None)
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8; OUT_LEN] {
        &self.0.as_bytes()
    }

    /// Create a `Hash` from its cbor representation.
    pub fn to_cbor(&self) -> Vec<u8> {
        let mut writer = Vec::new();
        into_writer(self, &mut writer).expect("Failed to serialize Hash");
        writer
        // let v = cbor!(self.clone()).expect("Failed to serialize Hash");
        // info!("Hash to_cbor: {:?}", v);
        // v.into_bytes().expect("Failed to convert Hash to bytes")
    }

    /// Create a `Hash` from its cbor representation.
    pub fn from_cbor(bytes: Vec<u8>) -> Self {
        ciborium::de::from_reader(bytes.as_slice()).expect("Failed to deserialize Hash")
    }

    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }

    pub fn from_bytes(bytes: &[u8; OUT_LEN], opts: Option<HashOpts>) -> Self {
        let bytes = *bytes;
        if let Some(opts) = opts {
            if let Some(key) = opts.key {
                return Hash(B3Hash::from_bytes(bytes), Some(Box::new(key)));
            }
        }
        Hash(B3Hash::from_bytes(bytes), None)
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(key) = &self.1 {
            write!(f, "{}|{}", key.to_hex(), self.to_hex())
        } else {
            write!(f, "{}", self.to_hex())
        }
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(key) = &self.1 {
            write!(f, "{}|{}", key.to_hex(), self.to_hex())
        } else {
            write!(f, "{}", self.to_hex())
        }
    }
}

impl Into<B3Hash> for Hash {
    fn into(self) -> B3Hash {
        self.0
    }
}

impl From<B3Hash> for Hash {
    fn from(hash: B3Hash) -> Self {
        Hash(hash, None)
    }
}

impl Into<RecordKey> for Hash {
    fn into(self) -> RecordKey {
        RecordKey::from(self.to_cbor())
    }
}

impl From<RecordKey> for Hash {
    fn from(key: RecordKey) -> Self {
        Self::from_cbor(key.to_vec())
    }
}

impl Into<Vec<u8>> for Hash {
    fn into(self) -> Vec<u8> {
        self.to_cbor()
    }
}

impl From<&[u8]> for Hash {
    fn from(value: &[u8]) -> Hash {
        Hash::from_cbor(value.to_vec())
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl PartialOrd for Hash {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.as_bytes().cmp(&other.0.as_bytes()))
    }
}

pub trait CustomHash {
    fn hash(&self) -> [u8; OUT_LEN];
}
