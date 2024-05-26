use libp2p::{relay, swarm::NetworkBehaviour, Swarm};

use crate::common;
use tracing::{error, info};

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub relay: relay::Behaviour,
    pub common: common::Behaviour,
}
