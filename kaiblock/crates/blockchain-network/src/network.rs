use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::{Peer, NetworkMessage, NetworkError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::mempool::Mempool;


use rand::seq::IteratorRandom;


pub struct Network{
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    pub mempool: Mempool,
}


impl Network{
    pub fn new() -> Self{
        Self {
            peers: Arc::new(HashMap::new()),
            mempool: Mempool::new(),
        }
    }

    pub async fn start_listener(&self, addr: &str) ->Result<(), NetworkError>{
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on {}", addr);
        loop{
            let (socket, peer_addr) = listener.accept().await?;
            println!("Accepted connection from {}", peer_addr);

            let peers = self.peers.clone();
            tokio::spawn(async move{
                if let Err(e) = Slt::handle_connection(socket, peers).await?{
                    eprintln!("Error handling connection from {}: {}", peer_addr, e);
                }
            });
        }
    }


    pub async fn handle_connection(mut socket: TcpStream, peers: Arc<RwLock<HashMap<String, Peer>>>) ->Result<(), NetworkError>{
        let mut buffer = vec![0; 1024];
        loop{
            let n = socket.read(&mut buffer).await?;
            if n == 0{
                break; // Connection closed
            }

            let msg: NetworkMessage = bincode::deserialize(&buffer[..n]).map_err(|e| NetworkError::DeserializationError(e.to_string()))?;
            println!("Received message: {:?}", msg);
            // Handle the message (e.g., broadcast to other peers)
        }
        Ok(())
    }


    pub async fn connect_to_peer(&self, addr: &str) ->Result<(), NetworkError>{
        let socket = TcpStream::connect(addr).await?;
        let peer = Peer::new(socket.peer_addr()?);
        self.peers.write().await.insert(addr.to_string(), peer);
        println!("Connected to peer {}", addr);
        Ok(())
    }


    pub async fn broadcast_message(&self, msg: &NetworkMessage) ->Result<(), NetworkError>{
        let peers = self.peers.read().await?;
        for (addr, peer) in peers.iter(){
            let mut socket = TcpStream::connect(addr).await?;
            let data = bincode::serialize(msg).map_err(|e| NetworkError::SerializationError(e.to_string()))?;
            socket.write_all(&data).await?;
            println!("Sent message to {}", addr);
        }
        Ok(())
    }


    pub async fn broadcast_transaction(&self, tx: &Transaction){
        if self.mempool.add_tx(tx.clone()).await {
            let msg = NetworkMessage::new_transaction(tx.clone());
            self.broadcast(&msg).await.unwrap();
        }
    }



//     What’s still possible to improve/expand later

// Secure connections (TLS or Noise protocol).

// Peer discovery over the Internet (instead of static bootstrap).

// Message TTL and deduplication to prevent loops.

// Batching transactions to reduce message overhead.

// Integration with mining/validator scheduling for real block creation.

    // Gossip protocol to randomly select peers for broadcasting
    pub async gossip_message(&self, msg: &etworkMessage, max_peers: usize) {
        let peers = self.peers.read().await;
        let mut rng = rand::thread_rng();
        let selected_peers = peers.values().choose_multiple(&mut rng, max_peers);
        for peer in selected_peers {
            if let Ok(mut socket) = TcpStream::connect(peer.addr).await {
                let data = bincode::serialize(msg).unwrap();
                if let Err(e) = socket.write_all(&data).await {
                    eprintln!("Failed to send message to {}: {}", peer.addr, e);
                }
            }
        }
    }
}
