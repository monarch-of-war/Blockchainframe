use crate::network::Network;
use crate::ledger::block::Block;
use crate::ledger::transaction::TransactionTrait;
use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc};

/// In-memory network stub. Simulates broadcasting in-memory.
#[derive(Clone)]
pub struct InMemoryNetwork<Tx>
where
    Tx: TransactionTrait + Clone + Send + Sync + 'static,
{
    tx_sender: broadcast::Sender<Tx>,
    block_sender: broadcast::Sender<Block<Tx>>,
}

impl<Tx> InMemoryNetwork<Tx>
where
    Tx: TransactionTrait + Clone + Send + Sync + 'static,
{
    pub fn new(buffer_size: usize) -> Self {
        let (tx_sender, _) = broadcast::channel(buffer_size);
        let (block_sender, _) = broadcast::channel(buffer_size);
        Self { tx_sender, block_sender }
    }
}

#[async_trait]
impl<Tx> Network<Tx> for InMemoryNetwork<Tx>
where
    Tx: TransactionTrait + Clone + Send + Sync + 'static,
{
    async fn broadcast_transaction(&self, tx: Tx) {
        let _ = self.tx_sender.send(tx); // ignore errors
    }

    async fn broadcast_block(&self, block: Block<Tx>) {
        let _ = self.block_sender.send(block);
    }

    async fn subscribe_transactions(&self) -> mpsc::Receiver<Tx> {
        let mut rx = self.tx_sender.subscribe();
        let (tx_out, rx_out) = mpsc::channel(32);
        tokio::spawn(async move {
            while let Ok(tx) = rx.recv().await {
                if tx_out.send(tx.clone()).await.is_err() {
                    break;
                }
            }
        });
        rx_out
    }

    async fn subscribe_blocks(&self) -> mpsc::Receiver<Block<Tx>> {
        let mut rx = self.block_sender.subscribe();
        let (tx_out, rx_out) = mpsc::channel(32);
        tokio::spawn(async move {
            while let Ok(block) = rx.recv().await {
                if tx_out.send(block.clone()).await.is_err() {
                    break;
                }
            }
        });
        rx_out
    }
}
