
pub mod types;
pub mod transaction;
pub mod block;
pub mod chain;
pub mod error;

// Re-export commonly used types
pub use types::{Hash, Amount, BlockHeight};
pub use transaction::{Transaction, TxInput, TxOutput, TxId};
pub use block::{Block, BlockHeader};
pub use chain::Blockchain;
pub use error::BlockchainError;

// Convenient Result type for this crate
pub type Result<T> = std::result::Result<T, BlockchainError>;