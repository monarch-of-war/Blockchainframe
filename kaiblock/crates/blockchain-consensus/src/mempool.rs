use blockchain_core::block::Block;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;


#[derive(Clone)]
pub struct Mempool {
    txs: Arc<RwLock<HashMap<Vec<u8>, Transaction>>>,

}


imp Mempool{

    pub fn new() -> Self {
        Self{
            txs: Arc::new(RwLock::new(ashMap::new())),
        }
    }


    pub async fn add_tx(&self, tx: Transaction) -> bool {
        let hash = tx.hash();
        let mut txs = self.txs.write().await;

        if txs.contains_key(&hash) {
            false
        }else{
            txs.insert(hash, tx);
            true
        }
    }


    pub async fn remove_tx(&self, hash: &[u8]) -> bool {
        let mut txs = self.txs.write().await;
        txs.remove(hash).is_some()
    }

    pub async get_all_txs(&self) -> Vec<Transaction> {
        let txs = self.txs.read().await;
        txs.values().cloned().collect()
    }


}


