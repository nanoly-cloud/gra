use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::{crate_name, crate_version};
use hashbrown::{HashMap, HashSet};
use lazy_static::lazy_static;
use libp2p::identity::Keypair;
use serde::{Deserialize, Serialize};

use tracing::debug;

use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use futures::StreamExt;

use libp2p::{
    core::Multiaddr,
    dcutr, identify, identity,
    kad::{self, store::MemoryStore},
    mdns,
    multiaddr::Protocol,
    noise, ping, relay,
    request_response::{self, cbor, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, StreamProtocol, Swarm, SwarmEvent},
    tcp, yamux, PeerId,
};

mod behaviour;
pub use behaviour::{Behaviour, BehaviourEvent};

pub mod command;
pub mod event;

use crate::hash::Hash;
use crate::models::{Block, BLOCK_SIZE};

pub const BUF_SIZE: usize = BLOCK_SIZE + 10;

lazy_static! {
    pub static ref PROTOCOL: String = format!("/{}/{}", crate_name!(), crate_version!());
}

#[repr(C)]
#[derive(Debug)]
pub enum Event {
    InboundRequest {
        request: BlockRequest,
        channel: ResponseChannel<BlockResponse>,
    },
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BlockRequest {
    pub hash: Hash,
}

impl BlockRequest {
    pub fn new(hash: Hash) -> Self {
        Self { hash }
    }

    pub fn inner(&self) -> &Hash {
        &self.hash
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BlockResponse {
    block: Block,
}

impl BlockResponse {
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    pub fn inner(&self) -> &Block {
        &self.block
    }
}

// impl Into<Vec<u8>> for BlockResponse {
//     fn into(self) -> Vec<u8> {
//         todo!()
//         // self.block.to_vec()
//     }
// }

pub fn generate_identity(secret_key: Option<&mut [u8]>) -> identity::Keypair {
    if let Some(seed) = secret_key {
        let mut hash = Hash::new(seed, None).as_bytes().clone();
        identity::Keypair::ed25519_from_bytes(hash).expect("only errors on wrong length")
    } else {
        identity::Keypair::generate_ed25519()
    }
}
