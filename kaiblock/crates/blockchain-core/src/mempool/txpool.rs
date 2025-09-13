use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ledger::transaction::TransactionTrait;
use super::MempoolError;

/// A simple in-memory transaction pool.
/// Production systems would shard, prioritize by fees, and limit capacity.
pub struct TxPool<T: TransactionTrait + Clone + Send + Sync + 'static> {
    inner: Arc<RwLock<HashMap<Vec<u8>, T>>>, // tx_hash -> tx
    seen: Arc<RwLock<HashSet<Vec<u8>>>>,     // deduplication
    capacity: usize,
}

impl<T: TransactionTrait + Clone + Send + Sync + 'static> TxPool<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            seen: Arc::new(RwLock::new(HashSet::new())),
            capacity,
        }
    }

    /// Insert a transaction into the pool.
    pub async fn insert(&self, tx: T) -> Result<(), MempoolError> {
        let hash = tx.hash();

        // check capacity
        let inner = self.inner.read().await;
        if inner.len() >= self.capacity {
            return Err(MempoolError::Invalid("pool is full".into()));
        }
        drop(inner);

        // deduplication
        {
            let mut seen = self.seen.write().await;
            if !seen.insert(hash.clone()) {
                return Err(MempoolError::Duplicate);
            }
        }

        let mut inner = self.inner.write().await;
        inner.insert(hash, tx);
        Ok(())
    }

    /// Fetch up to `max` transactions for block production.
    pub async fn fetch(&self, max: usize) -> Vec<T> {
        let inner = self.inner.read().await;
        inner.values().cloned().take(max).collect()
    }

    /// Remove transactions that were included in a block.
    pub async fn remove(&self, txs: &[T]) {
        let mut inner = self.inner.write().await;
        let mut seen = self.seen.write().await;

        for tx in txs {
            let hash = tx.hash();
            inner.remove(&hash);
            seen.remove(&hash);
        }
    }

    /// Current pool size
    pub async fn size(&self) -> usize {
        self.inner.read().await.len()
    }
}
