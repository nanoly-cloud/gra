use libp2p::{dcutr, swarm::NetworkBehaviour, Swarm};

pub async fn handle<B: NetworkBehaviour>(swarm: &mut Swarm<B>, event: &dcutr::Event) {
    match event {
        _ => {
            todo!("{event:?}");
        }
    }
}
