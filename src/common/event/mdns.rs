use crate::node;
use libp2p::{
    self, mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    Swarm,
};
use tracing::{info, warn};

pub(crate) async fn handle_event<B: NetworkBehaviour>(swarm: &mut Swarm<B>, event: &mdns::Event) {
    // Example of handling mDNS events and potentially modifying the swarm
    println!("Shared handler for mDNS event: {:?}", event);

    // Add logic that might involve modifying the swarm
    match event {
        mdns::Event::Discovered(peers) => {
            unimplemented!("Discovered peers: {peers:?}");
            peers.iter().for_each(|peer| {
                // swarm
                //     .behaviour_mut()
                //     .kad
                //     .add_address(&peer.0, peer.1.clone());
            });
        }
        mdns::Event::Expired(peers) => {
            unimplemented!("Expired peers: {peers:?}");
        }
        e => {
            unimplemented!("{e:?}");
        }
    }
}
