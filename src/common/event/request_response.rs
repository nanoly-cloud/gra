use libp2p::kad;
use libp2p::{
    self, request_response,
    swarm::{NetworkBehaviour, SwarmEvent},
    Swarm,
};
use tracing::{error, info, trace, warn};

use crate::common::{BlockRequest, BlockResponse};
use crate::hash::Hash;

pub async fn handle<B: NetworkBehaviour>(
    swarm: &mut Swarm<B>,
    event: &request_response::Event<BlockRequest, BlockResponse>,
) {
    match event {
        request_response::Event::Message { message, .. } => match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                // self.event_sender
                //     .send(Event::InboundRequest {
                //         request: request.into(),
                //         channel,
                //     })
                //     .await
                //     .expect("Event receiver not to be dropped.");
            }
            request_response::Message::Response {
                request_id,
                response,
            } => {
                // let _ = self
                //     .pending_request_file
                //     .remove(&request_id)
                //     .expect("Request to still be pending.")
                //     .send(Ok(response.into()));
            }
        },
        request_response::Event::OutboundFailure {
            request_id, error, ..
        } => {
            // let _ = self
            //     .pending_request_file
            //     .remove(&request_id)
            //     .expect("Request to still be pending.")
            //     .send(Err(anyhow!(error)));
        }
        request_response::Event::ResponseSent { .. } => {
            trace!("Response sent.");
        }
        request_response::Event::InboundFailure {
            peer,
            request_id,
            error,
        } => {
            error!(
                "Inbound request from {peer:?} failed: {error:?}",
                peer = peer,
                error = error
            );
        }
        e => {
            todo!("{e:?}");
        }
    }
}
