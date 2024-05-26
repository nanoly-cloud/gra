use anyhow::Result;
use libp2p::{
    identity::Keypair,
    relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    Swarm,
};
use tracing::{error, info};

use crate::common;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub relay_client: relay::client::Behaviour,
    pub common: common::Behaviour,
}

impl Behaviour {
    pub fn new(
        identity: &Keypair,
        daemon_behaviour: relay::client::Behaviour,
        bootnodes: Option<[&str; 1]>,
    ) -> Result<Self> {
        let common_behaviour = common::Behaviour::new(&identity, bootnodes)?;

        Ok(Self {
            relay_client: daemon_behaviour,
            common: common_behaviour,
        })
    }
}
