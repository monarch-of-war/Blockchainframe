use crate::peer::Peer;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct PeerDiscovery {
    pub peers: Arc<RwLock<HashMap<String, Peer>>>,

}

impl PeerDiscovery {
    pub fn new(peers: Arc<RwLock<HashMap<String, Peer>>>) -> Self {
        Self { peers }
    }

    pub async fn add_peer(&self, peer: Peer){
        let mut peers = self.peers.write().await;
        peers.insert(peer.addr.to_string(), peer);
    }

    pub async fn bootstrap(&self, bootstrap_peers: Vec<String>) {
        for add in bootstrap_peers {
            let peer = Peer::new(add.parse().unwrap());
            self.add_peer(peer).await;
        }
    }

    pub async fn get_peers(&self) -> Vec<Peer> {
        self.peers.read.await.values().cloned().collect()
    }
}
