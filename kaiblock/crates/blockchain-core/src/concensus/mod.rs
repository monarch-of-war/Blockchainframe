// consensus/mod.rs
//! Consensus abstractions for the framework.
//! Engines implement `ConsensusEngine` to plug into the node runtime.

//! Consensus module
//!
//! Defines a generic Consensus trait and exposes implementations
//! (e.g., Proof-of-Work).

pub mod difficulty;
pub mod pow;
pub mod pos;
pub mod consensus;

pub use pos::{PoSEngine, Validator, StakingState};
// pub use consensus::{ConsensusEngine, ConsensusError, BlockStore, ConsensusRunner, ProductionParams};
pub use crate::consensus::{BlockStore, ConsensusError, ProductionParams};

pub use crate::mempool::TxPool;

use async_trait::async_trait;
use thiserror::Error;

/// The core trait that all consensus mechanisms must implement.
/// This ensures pluggability across PoW, PoS, or custom algorithms.
pub trait Consensus {
    /// Verifies whether a block is valid under this consensus.
    ///
    /// # Arguments
    /// - `block_header`: serialized header (excluding consensus-specific fields).
    /// - `consensus_data`: extra data (e.g., nonce for PoW, signature for PoS).
    ///
    /// # Returns
    /// true if block passes consensus verification.
    fn verify(&self, block_header: &[u8], consensus_data: &[u8]) -> bool;

    /// Optional hook for mining/producing a block (PoW) or validating a slot (PoS).
    fn produce(&self, block_header: &[u8]) -> Option<Vec<u8>>;
}


// /// Simple result type for consensus operations
// pub type ConsensusResult<T> = Result<T, ConsensusError>;

#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("invalid block: {0}")]
    InvalidBlock(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("timeout")]
    Timeout,
}

/// Minimal information a consensus engine might need to start mining / producing blocks.
#[derive(Debug, Clone)]
pub struct ProductionParams{
    pub timestamp: u64,
    pub max_txs: usize,
    pub extra: Vec<u8>, // e.g., extra nonce space for PoW
}

/// Trait a block store must implement for the consensus engine to fetch/append blocks.
/// Keep this trait minimal: you can implement an adapter to your storage backend.
#[async_trait]
pub trait BlockStore<BlockT: Send + Sync> : Send + Sync {
    async fn tip_hash(&self) -> Option<Vec<u8>>;
    async fn get_block(&self, hash: &[u8]) -> Option<BlockT>;
    async fn append_block(&self, block: BlockT) -> Result<(), ConsensusError>;
}

/// Consensus engine trait. Engines must implement produce/validate hooks.
/// Generic over "BlockT" so it remains ledger-agnostic.
#[async_trait]
pub trait ConsensusEngine<BlockT: Send + Sync + 'static>: Send + Sync {
    /// Called to produce a block (mining / proposing).
    async fn produce_block(&self, params: ProductionParams) -> Result<BlockT, ConsensusError>;

    /// Called to validate and commit a block.
    async fn validate_and_commit(&self, block: BlockT) -> Result<(), ConsensusError>;

    /// Lightweight check for header proof validity (useful for gossip-level filtering).
    fn verify_header(&self, header_bytes: &[u8]) -> Result<(), ConsensusError>;
}

/// Convenience wrapper that holds an engine + blockstore and wires them together.
/// The Node runtime can use this to call produce/validate without knowing engine internals.
pub struct ConsensusRunner<E, B, BlockT>
where
    E: ConsensusEngine<BlockT>,
    B: BlockStore<BlockT>,
{
    pub engine: E,
    pub store: std::sync::Arc<B>,
}

impl<E, B, BlockT> ConsensusRunner<E, B, BlockT>
where
    E: ConsensusEngine<BlockT>,
    B: BlockStore<BlockT>,
{
    pub fn new(engine: E, store: std::sync::Arc<B>) -> Self {
        Self { engine, store }
    }

    /// Try producing a block and committing it atomically via validate_and_commit.
    /// The runtime can call this on a task loop.
    pub async fn produce_and_commit(&self, params: BlockProductionParams) -> ConsensusResult<()> {
        let block = self.engine.produce_block(params).await?;
        self.engine.validate_and_commit(block).await
    }
}
