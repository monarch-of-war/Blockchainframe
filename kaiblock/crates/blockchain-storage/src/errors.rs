#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("block not found")]
    NotFound,
    #[error("serialization error")]
    Serialization(#[from] bincode::Error),
    #[error("database error")]
    Database(#[from] sled::Error),
}
