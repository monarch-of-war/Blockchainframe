pub mod block;
pub mod transaction;
pub mod state;
pub mod mempool;
pub mod chain;
pub mod types;
pub mod validation;

use thiserror::Error;

/// Core blockchain errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BlockchainError {
    #[error("Cryptographic error: {0}")]
    CryptoError(#[from] blockchain_crypto::CryptoError),
    
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    
    #[error("Double spending detected: {0}")]
    DoubleSpending(String),
    
    #[error("Invalid chain: {0}")]
    InvalidChain(String),
    
    #[error("State error: {0}")]
    StateError(String),
    
    #[error("Mempool error: {0}")]
    MempoolError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, BlockchainError>;

// Re-export commonly used types
pub use block::{Block, BlockHeader, BlockBody};
pub use transaction::{Transaction, TransactionInput, TransactionOutput, UTXO};
pub use state::{AccountState, UTXOSet, WorldState};
pub use mempool::{Mempool, TransactionPool};
pub use chain::{Blockchain, ChainConfig};
pub use types::*;
pub use validation::{Validator, ValidationRules};

// Re-export crypto types for convenience
pub use blockchain_crypto::{
    Hash256, Address, AddressType, PublicKey, PrivateKey, 
    KeyPair, Signature, MerkleTree, MerkleProof
};