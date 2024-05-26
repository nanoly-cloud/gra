use libp2p::swarm::{NetworkBehaviour, Swarm};

use super::BehaviourEvent;

pub mod dcutr;
// pub mod mdns;
pub mod identify;
pub mod kad;
pub mod request_response;

pub async fn handle<B: NetworkBehaviour>(swarm: &mut Swarm<B>, event: BehaviourEvent) {
    match event {
        BehaviourEvent::Kad(event) => kad::handle(swarm, &event).await,
        BehaviourEvent::Identify(event) => identify::handle(swarm, &event).await,
        BehaviourEvent::RequestResponse(event) => request_response::handle(swarm, &event).await,
        BehaviourEvent::Dcutr(event) => {
            todo!("DCUTR {event:?}")
        }
        e => {
            todo!("{e:?}")
        }
    }
}
