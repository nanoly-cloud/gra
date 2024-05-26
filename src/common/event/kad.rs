use libp2p::kad;
use libp2p::{
    self, mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    Swarm,
};
use tracing::{error, info, warn};

use crate::hash::Hash;

pub async fn handle<B: NetworkBehaviour>(swarm: &mut Swarm<B>, event: &kad::Event) {
    match event {
        kad::Event::InboundRequest { request, .. } => match request {
            kad::InboundRequest::GetRecord {
                num_closer_peers,
                present_locally,
            } => {
                unimplemented!("GetRecord {num_closer_peers}, {present_locally}");
            }
            _ => {
                unimplemented!("{request:?}");
            }
        },
        kad::Event::OutboundQueryProgressed { id, result, .. } => match result {
            kad::QueryResult::GetClosestPeers(Ok(ok)) => {
                if ok.peers.is_empty() {
                    // self.send(Err(anyhow!(error)));
                }

                unimplemented!("Query finished with closest peers: {:#?}", ok.peers);
                // self.event_sender.send(Event::GetClosestPeers { peers: ok.peers }).await.expect("Event receiver not to be dropped."

                // return ok;
            }
            kad::QueryResult::GetClosestPeers(Err(kad::GetClosestPeersError::Timeout {
                ..
            })) => {
                error!("Query for closest peers timed out")
            }
            kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                key,
                providers,
                ..
            })) => {
                info!(
                    "Checking Providers for key {:?}",
                    std::str::from_utf8(key.as_ref()).unwrap()
                );
                for peer in providers {
                    info!(
                        "Peer {peer:?} provides key {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
            }
            kad::QueryResult::GetProviders(Ok(
                kad::GetProvidersOk::FinishedWithNoAdditionalRecord { closest_peers },
            )) => {
                unimplemented!("Finished with no additional record, {closest_peers:?}");
            }
            kad::QueryResult::GetProviders(Err(err)) => {
                unimplemented!("Failed to get providers: {err:?}");
            }
            kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                record: kad::Record { key, value, .. },
                ..
            }))) => {
                unimplemented!(
                    "Got record {:?} {:?}",
                    std::str::from_utf8(key.as_ref()).unwrap(),
                    std::str::from_utf8(&value).unwrap(),
                );
            }
            kad::QueryResult::GetRecord(Err(err)) => {
                unimplemented!("Failed to get record: {err:?}");
            }
            kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                unimplemented!(
                    "Successfully put record {:?}",
                    std::str::from_utf8(key.as_ref()).unwrap()
                );
                // let _ = self
                //     .swarm
                //     .behaviour_mut()
                //     .kad
                //     .start_providing(key.to_owned());
                info!(
                    "Successfully set {:?}",
                    std::str::from_utf8(key.as_ref()).unwrap()
                );
            }
            kad::QueryResult::PutRecord(Err(err)) => {
                error!("Failed to put record: {err:?}");
            }
            kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
                let hash = Hash::from(key.clone());
                info!("Successfully put provider record {hash:?}",);
            }
            kad::QueryResult::StartProviding(Err(err)) => {
                error!("Failed to put provider record: {err:?}");
            }
            kad::QueryResult::StartProviding(e) => {
                info!("Unimplemented: StartProviding {e:?}");
                // let sender: oneshot::Sender<()> = self
                //     .pending_start_providing
                //     .remove(&id)
                //     .expect("Completed query to be previously pending.");
                // let _ = sender.send(());
            }
            kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                providers,
                ..
            })) => {
                // if let Some(sender) = self.pending_get_providers.remove(&id) {
                //     sender.send(providers).expect("Receiver not to be dropped");

                //     // Finish the query. We are only interested in the first result.
                //     self.swarm
                //         .behaviour_mut()
                //         .kad
                //         .query_mut(&id)
                //         .unwrap()
                //         .finish();
                // }
            }
            _ => {}
        },
        kad::Event::RoutingUpdated {
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer,
        } => {
            info!("Routing updated: {peer:?} {is_new_peer} {addresses:?} {bucket_range:?} {old_peer:?}");
        }
        e => {
            warn!("Unimplemented: {:?}", e);
        }
    }
}
