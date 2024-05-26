use libp2p::kad;
use libp2p::{
    self, identify,
    swarm::{NetworkBehaviour, SwarmEvent},
    Swarm,
};
use tracing::{error, info, warn};

use crate::hash::Hash;

pub async fn handle<B: NetworkBehaviour>(swarm: &mut Swarm<B>, event: &identify::Event) {
    match event {
        identify::Event::Sent { peer_id } => {
            info!("Told relay its public address {peer_id}");
        }
        identify::Event::Received {
            info: identify::Info { observed_addr, .. },
            ..
        } => {
            info!(address=%observed_addr, "Relay told us our observed address");
            swarm.add_external_address(observed_addr.clone());
        }
        e => {
            todo!("{e:?}");
        }
    }
}
