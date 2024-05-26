use anyhow::{anyhow, bail, Result};
use async_std::task;
use clap::{crate_name, crate_version};
use futures::channel::mpsc::{self, Receiver, Sender};
use futures::channel::oneshot;
use futures::prelude::*;
use futures::{FutureExt, StreamExt};
use hashbrown::{hash_map, HashMap};
use lazy_static::lazy_static;
use libp2p::identity::Keypair;
use libp2p::kad::{Record, RecordKey};
use libp2p::multiaddr::Protocol;
use libp2p::request_response::{cbor, OutboundRequestId, ProtocolSupport, ResponseChannel};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{
    dcutr, identify, kad, mdns, noise, ping, relay, request_response, tcp, yamux, Multiaddr,
    PeerId, StreamProtocol, Swarm, SwarmBuilder,
};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashSet;
use std::fmt::{self, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, error, info, trace};

use crate::common::{self, generate_identity, BlockRequest, BlockResponse};
use crate::hash::Hash;
use crate::models::Block;

mod behaviour;
use behaviour::{Behaviour, BehaviourEvent};

mod event;

mod client;
pub use client::Client;

mod command;
pub use command::Command;

lazy_static! {
    pub static ref PROTOCOL: String = format!("/gra/{}/{}", crate_name!(), crate_version!());
}

#[repr(C)]
pub struct Node {
    address: Multiaddr,
    daemon_address: Option<Multiaddr>,
    identity: Keypair,
    swarm: Swarm<Behaviour>,
    command_sender: Sender<command::Command>,
    command_receiver: Receiver<command::Command>,
    event_sender: Sender<common::Event>,
    event_receiver: Receiver<common::Event>,
    // TODO: Rename
    // TODO: Change to a more efficient data structure.
    pending: Pending,
}

#[derive(Debug, Default)]
pub struct Pending {
    peers: HashMap<PeerId, oneshot::Sender<HashSet<PeerId>>>,
    blocks: HashMap<Hash, oneshot::Sender<Result<()>>>,
    entries: HashMap<Hash, oneshot::Sender<Result<()>>>,
    dial: HashMap<PeerId, oneshot::Sender<Result<()>>>,
    start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
    get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
    request_file: HashMap<request_response::OutboundRequestId, oneshot::Sender<Result<Vec<u8>>>>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("identity", &self.identity)
            .field("deaemon_address", &self.daemon_address)
            // .field("inventory_filter", &self.inventory_filter)
            // .field("entries", &self.entries)
            // .field("swarm", &self.swarm)
            .finish()
    }
}

impl Node {
    pub fn new(
        address: Multiaddr,
        identity: Keypair,
        daemon_address: Option<Multiaddr>,
        bootnodes: Option<[&str; 1]>,
    ) -> Result<Self> {
        debug!("Creating Node");

        let peer_id = identity.public().to_owned().to_peer_id();
        info!("Peer ID: {:?}", peer_id);

        let mut swarm = SwarmBuilder::with_existing_identity(identity.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_dns()?
            .with_relay_client(noise::Config::new, yamux::Config::default)?
            .with_behaviour(|keypair, daemon_behaviour| {
                match Behaviour::new(keypair, daemon_behaviour, bootnodes) {
                    Ok(b) => b,
                    Err(e) => {
                        panic!("{e:?}")
                    }
                }
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        if let Some(daemon_address) = daemon_address.clone() {
            info!("Dialing daemon at {:?}", daemon_address);
            swarm.dial(daemon_address.clone())?;
        }

        let (command_sender, command_receiver) = mpsc::channel(0);
        let (event_sender, event_receiver) = mpsc::channel(0);

        Ok(Self {
            address,
            identity,
            daemon_address,
            swarm,
            command_sender,
            command_receiver,
            event_sender,
            event_receiver,
            pending: Default::default(),
        })
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                e = self.swarm.select_next_some() => crate::node::handle_swarm_event(self, e).await,
                cmd = self.command_receiver.next() => match cmd {
                    Some(c) => command::handle(self, c).await,
                    None=>  {
                        info!("Command channel closed, thus shutting down the network event loop.");
                        break;
                    },
                },
            }
        }
    }

    pub fn client(&self) -> Client {
        Client::new(self.command_sender.clone())
    }
}

pub async fn handle_swarm_event(node: &mut Node, event: SwarmEvent<BehaviourEvent>) {
    match event {
        SwarmEvent::ListenerClosed {
            addresses,
            listener_id,
            reason,
        } => {
            info!("ListenerClosed: {listener_id:?} {addresses:?} {reason:?}");
        }
        SwarmEvent::ListenerError { listener_id, error } => {
            error!("{listener_id} {error:?}");
        }
        SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
            info!(peer=%peer_id, address=%address, "Learned external address of peer");
        }
        SwarmEvent::NewExternalAddrCandidate { address } => {
            info!(address=%address, "Learned external address candidate");
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            let local_peer_id = node.swarm.local_peer_id();
            info!(
                "Local node is listening on {:?}",
                address.with(Protocol::P2p(local_peer_id.clone()))
            );
        }
        SwarmEvent::IncomingConnection {
            connection_id,
            local_addr,
            send_back_addr,
        } => {
            trace!("Incoming connection {connection_id:?} from {local_addr:?} {send_back_addr:?}");
        }
        SwarmEvent::ConnectionEstablished {
            peer_id, endpoint, ..
        } => {
            debug!("ConnectionEstablished: {peer_id:?} {endpoint:?}");
            if endpoint.is_dialer() {
                if let Some(sender) = node.pending.dial.remove(&peer_id) {
                    let _ = sender.send(Ok(()));
                }
            }
        }
        SwarmEvent::ConnectionClosed {
            peer_id,
            endpoint,
            cause,
            ..
        } => {
            debug!("ConnectionClosed: {peer_id:?} {endpoint:?} {cause:?}");
        }
        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
            error!("OutgoingConnectionError: {:?} {:?}", peer_id, error);
            if let Some(peer_id) = peer_id {
                if let Some(sender) = node.pending.dial.remove(&peer_id) {
                    let _ = sender.send(Err(anyhow!(error)));
                }
            }
        }
        SwarmEvent::IncomingConnectionError {
            local_addr,
            connection_id,
            send_back_addr,
            error,
        } => {
            error!("IncomingConnectionError: {local_addr:?} {connection_id:?} {send_back_addr:?} {error:?}");
        }
        SwarmEvent::Dialing {
            peer_id: Some(peer_id),
            ..
        } => trace!("Dialing {peer_id}"),
        SwarmEvent::Behaviour(event) => event::handle(&mut node.swarm, event).await,
        e => info!("{e:?}"),
    };
}
