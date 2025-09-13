// blockchain-core/src/network/mod.rs
//! Network abstraction for blockchain nodes.
//! Provides broadcasting & subscription for blocks and transactions.

pub mod in_memory;

use async_trait::async_trait;
use crate::ledger::block::Block;
use crate::ledger::transaction::TransactionTrait;

/// Generic network interface.
/// Developers can implement their own transport (TCP, WebSocket, libp2p, etc.)
#[async_trait]
pub trait Network<Tx>
where
    Tx: TransactionTrait + Send + Sync + 'static,
{
    /// Broadcast a transaction to peers
    async fn broadcast_transaction(&self, tx: Tx);

    /// Broadcast a block to peers
    async fn broadcast_block(&self, block: Block<Tx>);

    /// Subscribe to incoming transactions
    async fn subscribe_transactions(&self) -> tokio::sync::mpsc::Receiver<Tx>;

    /// Subscribe to incoming blocks
    async fn subscribe_blocks(&self) -> tokio::sync::mpsc::Receiver<Block<Tx>>;
}
