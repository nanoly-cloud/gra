use anyhow::{bail, Result};
use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use futures::StreamExt;
use libp2p::{request_response::ResponseChannel, Multiaddr, PeerId};
use std::collections::HashSet;
use tracing::{debug, info};

use crate::common::BlockResponse;
use crate::hash::Hash;
use crate::models::Block;

use super::command::Command;

#[derive(Clone)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    pub fn new(sender: mpsc::Sender<Command>) -> Self {
        Self { sender }
    }

    /// Listen for incoming connections on the given address.
    pub async fn start_listening(&mut self, address: Multiaddr) -> Result<()> {
        info!("Starting to listen on {:?}", address);
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening {
                address: address.to_owned(),
                sender,
            })
            .await?;
        info!("Listening on {:?}", address);
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Dial the given peer at the given address.
    pub async fn dial(&mut self, peer_id: PeerId, peer_addr: Multiaddr) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Advertise the local node as the provider of the given block on the DHT.
    pub async fn start_providing(&mut self, hash: Hash) {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartProviding {
                hash: Hash::new(hash.as_bytes(), None),
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.");
    }

    /// Find the providers for the given block on the DHT.
    pub async fn get_providers(&mut self, hash: Hash) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        debug!("Requesting providers for {:?}", hash);
        self.sender
            .send(Command::GetProviders {
                hash: Hash::new(hash.as_bytes(), None),
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        debug!("Waiting for providers for {:?}", hash);
        let peers = receiver.await.expect("Sender not to be dropped.");
        debug!("Received providers for {:?}", hash);
        peers
    }

    /// Request the content of the given block from the given peer.
    pub async fn request_block(
        &mut self,
        hash: Hash,
        peers: Option<HashSet<PeerId>>,
    ) -> Result<Block> {
        let peers = if let Some(peers) = peers {
            peers
        } else {
            self.get_providers(hash.clone()).await
        };

        let result: Vec<Block> = Vec::new();

        for peer in peers {
            let (sender, receiver) = oneshot::channel();
            self.sender
                .send(Command::RequestBlock {
                    hash: hash.clone(),
                    peer,
                    sender,
                })
                .await
                .expect("Command receiver not to be dropped.");

            let res = receiver.await.expect("Sender not be dropped.");
        }
        // result.sort_by_cached_key(|block| block.);
        if result.is_empty() {
            bail!("No block found for {:?}", hash);
        }
        // TODO: Add logic, to select the best block from the list of results.
        // TODO: Add logic to handle Block enum
        Ok(result[0].clone())
    }

    /// Respond with the provided block content to the given request.
    pub async fn respond_block(&mut self, block: Vec<u8>, channel: ResponseChannel<BlockResponse>) {
        self.sender
            .send(Command::RespondBlock {
                block: block.as_slice().into(),
                channel,
            })
            .await
            .expect("Command receiver not to be dropped.");
    }
}
