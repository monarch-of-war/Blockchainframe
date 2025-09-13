use sled::Db;
use crate::errors::StorageError;
use blockchain_core::block::Block; // <-- Correct import path
use bincode;

pub struct SledBlockStore {
    db: Db,
}

impl SledBlockStore {
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn hash_key(hash: &[u8]) -> Vec<u8> {
        let mut key = b"hash:".to_vec();
        key.extend_from_slice(hash);
        key
    }

    pub fn height_key(height: u64) -> Vec<u8> {
        let mut key = b"height:".to_vec();
        key.extend_from_slice(&height.to_be_bytes());
        key
    }

    pub fn serialize_block(block: &Block) -> Result<Vec<u8>, StorageError> {
        Ok(bincode::serialize(block)?)
    }

    pub fn deserialize_block(data: &[u8]) -> Result<Block, StorageError> {
        Ok(bincode::deserialize(data)?)
    }


    pub async get_block_by_hash(&self, hash: &[u8]
    ) -> Result<Option<Block>, StorageError>{
        match self.db.get(Self::hash_key(hash))? {
            Some(data) => Ok(Some(Self::deserialize_block(&data)?)),
            None => Ok(None),
        }
    }

    pub async get_block_by_height(&self, height: u64
    ) -> Result<Option<Block>, StorageError>{
        match self.db.get(Self::height_key(height))? {
            Some(data) => Ok(Some(Self::deserialize_block(&data)?)),
            None => Ok(None),
        }
    }

    pub async get_latest_block(&self) -> Result<Option<Block>, StorageError>{
        let mut latest: Option<u64> = None;
        for entry in self.db.scan_prefix(b"height:") {
            let (_, value) = entry?;
            let block = Self::deserialize_block(&value)?;

            if latest.is_none() || block.height > latest.as_ref().unwrap().height {
                latest = Some(block);
            }

        }

        Ok(latest)
    }
}


