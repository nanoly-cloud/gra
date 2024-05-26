use anyhow::{anyhow, Error, Result};
use futures::channel::oneshot;
use hashbrown::hash_map;
use libp2p::core::transport::ListenerId;
use libp2p::multiaddr::Protocol;
use libp2p::Swarm;
use libp2p::{request_response::ResponseChannel, Multiaddr, PeerId};
use std::collections::HashSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use tracing::info;

use crate::common::{BlockRequest, BlockResponse};
use crate::hash::Hash;
use crate::models::Block;
use crate::node::Node;

use super::Behaviour;

#[derive(Debug)]
pub enum Command {
    StartListening {
        address: Multiaddr,
        sender: oneshot::Sender<Result<()>>,
    },
    GetPeers {
        peer_id: PeerId,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<()>>,
    },
    StartProviding {
        hash: Hash,
        sender: oneshot::Sender<()>,
    },
    GetProviders {
        hash: Hash,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    RequestBlock {
        hash: Hash,
        peer: PeerId,
        sender: oneshot::Sender<Result<Vec<u8>>>,
    },
    RespondBlock {
        block: Block,
        channel: ResponseChannel<BlockResponse>,
    },
}

pub async fn handle(node: &mut Node, command: Command) {
    let swarm = &mut node.swarm;
    match command {
        Command::GetPeers { peer_id, sender } => {
            unimplemented!("Search for the closest peers to {peer_id}");
        }
        Command::StartListening { address, sender } => {
            let mut addresses: Vec<Multiaddr> = Vec::new();
            let protocol_stack = address.protocol_stack();

            if protocol_stack.count() == 1 {
                addresses.push(
                    address
                        .to_owned()
                        .with(libp2p::multiaddr::Protocol::Udp(0))
                        .with(libp2p::multiaddr::Protocol::QuicV1),
                );
                addresses.push(address.to_owned().with(libp2p::multiaddr::Protocol::Tcp(0)));
            } else {
                addresses.push(address.to_owned());
            }

            let mut listeners: Vec<ListenerId> = Vec::new();
            let mut error: Option<Error> = None;
            'listen_on: for address in addresses {
                let res = match swarm.listen_on(address) {
                    Ok(listener_id) => listener_id,
                    Err(e) => {
                        error = Some(anyhow!(e));
                        break 'listen_on;
                    }
                };
                listeners.push(res);
            }

            for listener_id in listeners {
                swarm.remove_listener(listener_id);
            }

            if let Some(error) = error {
                sender.send(Err(error));
            } else {
                sender.send(Ok(()));
            }
        }
        Command::Dial {
            peer_id,
            peer_addr,
            sender,
        } => {
            if let hash_map::Entry::Vacant(e) = node.pending.dial.entry(peer_id) {
                match swarm.dial(peer_addr.with(Protocol::P2p(peer_id))) {
                    Ok(()) => {
                        e.insert(sender);
                    }
                    Err(e) => {
                        let _ = sender.send(Err(anyhow!(e)));
                    }
                }
            } else {
                todo!("Already dialing peer.");
            }
        }
        Command::StartProviding { hash, sender } => {
            let query_id = swarm
                .behaviour_mut()
                .common
                .kad
                .start_providing(hash.to_cbor().into())
                .expect("No store error.");
            node.pending.start_providing.insert(query_id, sender);
        }
        Command::GetProviders { hash, sender } => {
            let query_id = swarm.behaviour_mut().common.kad.get_providers(hash.into());
            node.pending.get_providers.insert(query_id, sender);
        }
        Command::RequestBlock { hash, peer, sender } => {
            let request_id = swarm
                .behaviour_mut()
                .common
                .request_response
                .send_request(&peer, BlockRequest { hash });
            node.pending.request_file.insert(request_id, sender);
        }
        Command::RespondBlock { block, channel } => {
            swarm
                .behaviour_mut()
                .common
                .request_response
                .send_response(channel, BlockResponse::new(block))
                .expect("Connection to peer to be still open.");
        }
    }
}
