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
    PeerId, StreamProtocol, Swarm,
};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashSet;
use std::fmt::{self, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, error, info, trace};

use crate::common::{self, generate_identity, BlockRequest, BlockResponse, Event};
use crate::hash::Hash;
use crate::models::Block;

pub mod behaviour;
use behaviour::{Behaviour, BehaviourEvent};

pub mod event;

lazy_static! {
    pub static ref PROTOCOL: String = format!("/gra/{}/{}", crate_name!(), crate_version!());
}

#[repr(C)]
pub struct Daemon {
    address: Multiaddr,
    identity: Keypair,
    swarm: Swarm<Behaviour>,
    event_receiver: Receiver<Event>,
    event_sender: Sender<Event>,
    // TODO: Change to a more efficient data structure.
    pending: Pending,
}

#[derive(Debug, Default)]
pub struct Pending {
    dial: HashMap<PeerId, oneshot::Sender<Result<()>>>,
    start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
    get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
    request_file: HashMap<request_response::OutboundRequestId, oneshot::Sender<Result<Vec<u8>>>>,
}

impl fmt::Debug for Daemon {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Daemon")
            .field("identity", &self.identity)
            .field("deaemon_address", &self.address)
            // .field("inventory_filter", &self.inventory_filter)
            // .field("entries", &self.entries)
            // .field("swarm", &self.swarm)
            .finish()
    }
}

impl Daemon {
    pub fn new(address: Multiaddr, identity: Keypair) -> Result<Self> {
        debug!("Creating Daemon...");

        let peer_id = identity.public().clone().to_peer_id();

        let common_behaviour = common::Behaviour::new(&identity, None)?;

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_dns()?
            .with_behaviour(|keypair| Behaviour {
                relay: relay::Behaviour::new(peer_id, Default::default()),
                common: common_behaviour,
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // swarm
        //     .behaviour_mut()
        //     .common
        // .kad
        // .set_mode(Some(kad::Mode::Server));

        // swarm.behaviour_mut().kad.bootstrap();

        let (event_sender, event_receiver) = mpsc::channel(0);
        // let event_loop = EventLoop::<Behaviour>::new(swarm, command_receiver, event_sender);

        Ok(Self {
            address,
            identity,
            swarm,
            event_sender,
            event_receiver,
            pending: Default::default(),
        })
    }

    pub async fn run(mut self) {
        loop {
            // TODO: Either handle commands or remove the select
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
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
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    // info!(peer=%peer_id, "Outgoing connection error: {error}");
                    error!("{peer_id} Outgoing connection error: {error}");
                } else {
                    info!("Outgoing connection error: {error}");
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                info!(
                    "Local node is listening on {:?}",
                    address // .with(Protocol::P2p(local_peer_id))
                );
            }
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending.dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending.dial.remove(&peer_id) {
                        let _ = sender.send(Err(anyhow!(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                ..
            } => trace!("Dialing {peer_id}"),
            SwarmEvent::Behaviour(event) => self::event::handle(&mut self.swarm, event).await,
            e => info!("{e:?}"),
        };
    }
}

//      kad::QueryResult::GetProviders(Ok(
//          kad::GetProvidersOk::FoundProviders { key, providers, .. },
//      )) => {
//          for peer in providers {
//              println!(
//                  "Peer {peer:?} provides key {:?}",
//                  std::str::from_utf8(key.as_ref()).unwrap()
//              );
//          }
//      }
//      kad::QueryResult::GetProviders(Err(err)) => {
//          eprintln!("Failed to get providers: {err:?}");
//      }
//      kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
//          println!(
//              "Successfully put record {:?}",
//              std::str::from_utf8(key.as_ref()).unwrap()
//          );
//      }
//      kad::QueryResult::PutRecord(Err(err)) => {
//          eprintln!("Failed to put record: {err:?}");
//      }
//      kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
//          println!(
//              "Successfully put provider record {:?}",
//              std::str::from_utf8(key.as_ref()).unwrap()
//          );
//      }
//      kad::QueryResult::StartProviding(Err(err)) => {
//          eprintln!("Failed to put provider record: {err:?}");
//      }
