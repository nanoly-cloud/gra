use super::{Behaviour, BehaviourEvent, Node};
use tracing::{error, info};

use crate::common;

use libp2p::{relay, Swarm};

pub async fn handle(swarm: &mut Swarm<Behaviour>, event: BehaviourEvent) {
    match event {
        BehaviourEvent::RelayClient(relay::client::Event::ReservationReqAccepted {
            relay_peer_id,
            renewal,
            limit,
        }) => {
            info!(peer=%relay_peer_id, renewal=%renewal, "Reservation Request Accepted. {limit:#?} slots reserved.");
        }
        BehaviourEvent::RelayClient(event) => {
            info!(?event)
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
