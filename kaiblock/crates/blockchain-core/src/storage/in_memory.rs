// storage/in_memory.rs
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::consensus::{BlockStore, ConsensusError};
use crate::ledger::block::Block;
use crate::ledger::transaction::TransactionTrait;

// The InMemoryBlockStore implements BlockStore<Block<Tx>> where Block<Tx> 
//is your ledger::block::Block<T>. It uses Block::hash() as the block key — 
//that method was implemented earlier in your block.rs.
// The store is async and uses RwLock, matching consensus and genesis usage.
// append_block returns Result<(), ConsensusError> and is already compatible 
//with genesis::build_and_persist_genesis and the PoS engine's validate_and_commit 
//(which calls append_block(...).await).
// This in-memory store is meant for tests, CI, and single-node demos. 
//For production you’ll later implement a persistent RocksDbBlockStore / 
// that also implements the same BlockStore trait.
// If you used slightly different trait paths or names in your local files, 
//adapt imports: crate::consensus::BlockStore and crate::consensus::ConsensusError 
//must be visible where in_memory.rs lives (they were defined in consensus/mod.rs earlier).




/// In-memory BlockStore implementation suitable for tests and single-node runs.
/// Thread-safe and clonable (internally wrapped by Arc).
#[derive(Clone)]
pub struct InMemoryBlockStore<Tx>
where
    Tx: TransactionTrait + Clone + Send + Sync + 'static,
{
    inner: Arc<InMemoryBlockStoreInner<Tx>>,
}

struct InMemoryBlockStoreInner<Tx>
where
    Tx: TransactionTrait + Clone + Send + Sync + 'static,
{
    /// map: block_hash -> block
    blocks: RwLock<HashMap<Vec<u8>, Block<Tx>>>,
    /// tip block hash
    tip: RwLock<Option<Vec<u8>>>,
}

impl<Tx> InMemoryBlockStore<Tx>
where
    Tx: TransactionTrait + Clone + Send + Sync + 'static,
{
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InMemoryBlockStoreInner {
                blocks: RwLock::new(HashMap::new()),
                tip: RwLock::new(None),
            }),
        }
    }

    /// Convenient helper: return the number of stored blocks.
    pub async fn len(&self) -> usize {
        let map = self.inner.blocks.read().await;
        map.len()
    }

    /// Return all blocks (cloned). Useful for debugging / tests.
    pub async fn all_blocks(&self) -> Vec<Block<Tx>> {
        let map = self.inner.blocks.read().await;
        map.values().cloned().collect()
    }
}

#[async_trait]
impl<Tx> BlockStore<Block<Tx>> for InMemoryBlockStore<Tx>
where
    Tx: TransactionTrait + Clone + Send + Sync + 'static,
{
    /// Return the current tip hash (if any).
    async fn tip_hash(&self) -> Option<Vec<u8>> {
        let tip = self.inner.tip.read().await;
        tip.clone()
    }

    /// Retrieve a block by its hash (cloned).
    async fn get_block(&self, hash: &[u8]) -> Option<Block<Tx>> {
        let map = self.inner.blocks.read().await;
        map.get(hash).cloned()
    }

    /// Append a block. Uses the block.hash() as the key and updates tip.
    async fn append_block(&self, block: Block<Tx>) -> Result<(), ConsensusError> {
        let hash = block.hash();
        let mut map = self.inner.blocks.write().await;
        // insert (overwrites if same hash - fine for tests)
        map.insert(hash.clone(), block);
        // update tip
        let mut tip = self.inner.tip.write().await;
        *tip = Some(hash);
        Ok(())
    }
}
