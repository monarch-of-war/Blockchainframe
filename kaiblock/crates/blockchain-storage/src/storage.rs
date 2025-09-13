use async_trait::async_trait;
use crate::block_store::SledBlockStore;
use crate::errors::StorageError;
use blockchain_core::block::Block; // <-- Matches your blockchain-core path

#[async_trait]
pub trait Storage {
    async fn save_block(&self, block: &Block) -> Result<(), StorageError>;
    async fn get_block_by_hash(&self, hash: &[u8]) -> Result<Option<Block>, StorageError>;
    async fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, StorageError>;
    async fn latest_block(&self) -> Result<Option<Block>, StorageError>;
}
