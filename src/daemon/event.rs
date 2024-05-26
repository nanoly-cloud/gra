use crate::common;

use super::{Behaviour, BehaviourEvent};
use libp2p::swarm::Swarm;

use tracing::{error, info};

pub async fn handle(swarm: &mut Swarm<Behaviour>, event: BehaviourEvent) {
    match event {
        BehaviourEvent::Relay(event) => {
            info!("Relay Event: {event:?}");
        }
        BehaviourEvent::Common(event) => {
            common::event::handle(swarm, event).await;
        }
        _ => {
            error!("Unhandled event: {event:?}");
            return;
        }
    };
}
