use sled::Db;
use crate::errors::StorageError;

pub struct StateStore {
    db: Db,
}

impl StateStore {
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.db.insert(key, value)?;
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.db.get(key)?.map(|v| v.to_vec()))
    }
}
