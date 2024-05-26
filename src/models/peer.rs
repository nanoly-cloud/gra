use super::Confidence;
use hashbrown::{hash_map::IntoIter, HashMap};
use libp2p::PeerId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Peer(PeerId, Confidence);

// #[derive(Debug, Default, Clone, PartialEq, Eq)]
// pub struct Peers(HashMap<PeerId, Confidence>);

// impl Peers {
//     // pub fn new() -> Self {
//     //     Peers(HashMap::new())
//     // }

//     pub fn inner(&self) -> &HashMap<PeerId, Confidence> {
//         &self.0
//     }

//     pub fn read(&self, loc: &PeerId) -> Option<&Confidence> {
//         self.0.get(loc)
//     }

//     pub fn write(&mut self, loc: &PeerId, confidence: Confidence) {
//         self.0.insert(loc.clone(), confidence);
//     }

//     pub fn remove(&mut self, loc: &PeerId) -> Option<Confidence> {
//         self.0.remove(loc)
//     }

//     // pub fn confidence(&self, peer: &PeerId) -> Confidence {
//     //     if let Some(confidence) = self.0.get(peer) {
//     //         confidence.clone()
//     //     } else {
//     //         0
//     //     }
//     // }
// }

// impl IntoIterator for Peers {
//     type Item = (PeerId, Confidence);
//     type IntoIter = IntoIter<PeerId, Confidence>;
//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }
