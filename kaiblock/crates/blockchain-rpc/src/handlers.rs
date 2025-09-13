use blockchain_storage::SledBlockStore;
use blockchain_network::Network;
use blockchain_core::{block: Block, transaction: Transaction};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::errors::RpcError;



#[derive(Clone)]
pub struct Rcpandler{
    pub store: Arc<RwLock<SledBlockStore>>,
    pub network: Arc<Network>,
}

imp Rcpandler{
    pub new(store: Arc<RwLock<SledBlockStore>>, network: Arc<Network>) -> Self {
        Self { store, network }
    }


    pub async get_block_by_height(&self, height: u64) -> Result<Block, RpcError> {
        let block = self.store.read().await.get_block_by_height(height).await
        .map_err(|_| RpcError::InternalServerError)?;
        block.ok_or(RpcError::BlockNotFound)

    }


    pub async fn get_latest_block(&self) ->Result<Block, RpcError> {
        let block = self.store.read().await.get_latest_block().await
        .map_err(|_| RpcError::InternalServerError)?;
        block.ok_or(RpcError::BlockNotFound)
    }

    pub async fn get_transaction(&self, tx: Transaction) -> Result<(), RpcError>{
        self.network.send_transaction(tx).await
        Ok(())
    }

    pub async fn get_mempool(&self) ->Vec<Transaction>{
        self.network.get_all_txs().await
    }


}


