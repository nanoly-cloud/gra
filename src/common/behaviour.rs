use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::{crate_name, crate_version};
use hashbrown::{hash_map, HashMap, HashSet};
use lazy_static::lazy_static;
use libp2p::identity::Keypair;
use serde::{Deserialize, Serialize};

use tracing::{debug, error, info, trace, warn};

use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use futures::StreamExt;

use libp2p::{
    self,
    core::Multiaddr,
    identity,
    kad::store::MemoryStore,
    multiaddr::Protocol,
    noise, relay,
    request_response::{cbor, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, StreamProtocol, Swarm, SwarmEvent},
    tcp, yamux, PeerId,
};

use crate::common::PROTOCOL;
use crate::common::{BlockRequest, BlockResponse};
use crate::hash::Hash;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub dcutr: libp2p::dcutr::Behaviour,
    pub identify: libp2p::identify::Behaviour,
    pub kad: libp2p::kad::Behaviour<MemoryStore>,
    pub request_response: libp2p::request_response::cbor::Behaviour<BlockRequest, BlockResponse>,
}

impl Behaviour {
    pub fn new(identity: &Keypair, bootnodes: Option<[&str; 1]>) -> Result<Self> {
        let key = identity.to_owned();
        let peer_id = key.public().to_peer_id();
        let store = MemoryStore::new(peer_id.clone());

        // let mdns_behaviour =
        //     libp2p::mdns::Behaviour::new(libp2p::mdns::Config::default(), peer_id)?;

        let mut cfg = libp2p::kad::Config::default();
        cfg.set_protocol_names(vec![StreamProtocol::new("/gra/kad/1.0.0")]);
        cfg.set_query_timeout(Duration::from_secs(5 * 60));

        let store = libp2p::kad::store::MemoryStore::new(peer_id);
        let mut kad_behaviour = libp2p::kad::Behaviour::with_config(peer_id, store, cfg);
        if let Some(bootnodes) = bootnodes {
            kad_behaviour.set_mode(None);

            for peer in bootnodes.iter() {
                kad_behaviour.add_address(
                    &peer.parse()?,
                    "/ip4/192.168.0.195/udp/58008/quic-v1".parse()?,
                );
            }
        } else {
            kad_behaviour.set_mode(Some(libp2p::kad::Mode::Server));
        }

        Ok(Self {
            identify: libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                format!("/gra/identify/{}", crate_version!()),
                identity.public(),
            )),
            dcutr: libp2p::dcutr::Behaviour::new(peer_id),
            kad: kad_behaviour,
            request_response: cbor::Behaviour::<BlockRequest, BlockResponse>::new(
                [(
                    StreamProtocol::new(PROTOCOL.as_str()),
                    ProtocolSupport::Full,
                )],
                libp2p::request_response::Config::default(),
            ),
        })
    }
}

// Assume you have a generic `Swarm` type that wraps your behaviour
