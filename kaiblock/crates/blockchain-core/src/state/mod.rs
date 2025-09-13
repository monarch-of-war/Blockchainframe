// state/mod.rs
//! Generic state trait and re-exports for in-memory account state.

pub mod in_memory;

pub use in_memory::{InMemoryState, AccountStateError};

use async_trait::async_trait;
use std::sync::Arc;

/// Address type alias — keep it consistent with your address type in blockchain-crypto.
pub type Address = Vec<u8>;

/// Result alias for state operations.
pub type StateResult<T> = Result<T, AccountStateError>;

/// Minimal state trait for an account-model chain.
/// This trait is intentionally small so it is easy to replace with a db-backed adapter.
#[async_trait]
pub trait State: Send + Sync {
    /// Get the balance for an address.
    async fn get_balance(&self, addr: &Address) -> StateResult<u128>;

    /// Get nonce for an address.
    async fn get_nonce(&self, addr: &Address) -> StateResult<u64>;

    /// Apply an account transfer (sender -> recipient, value).
    /// Should validate sender has sufficient balance and the nonce is correct.
    /// Returns Ok(()) on success; otherwise returns AccountStateError.
    async fn apply_transfer(&self, sender: &Address, recipient: &Address, value: u128, expected_nonce: u64) -> StateResult<()>;

    /// Convenience to credit an address (used for genesis & staking rewards).
    async fn credit(&self, addr: &Address, value: u128) -> StateResult<()>;

    /// Convenience to set nonce (used for genesis).
    async fn set_nonce(&self, addr: &Address, nonce: u64) -> StateResult<()>;
}
