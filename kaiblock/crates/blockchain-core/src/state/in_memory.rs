// state/in_memory.rs
//! In-memory account state: balances + nonces.
//! Simple, thread-safe, intended for tests and single-node runs.

use crate::state::{Address, State, StateResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum AccountStateError {
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("nonce mismatch: expected {expected}, found {found}")]
    NonceMismatch { expected: u64, found: u64 },
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Default)]
pub struct InMemoryState {
    inner: Arc<RwLock<InMemoryStateInner>>,
}

#[derive(Debug, Default)]
struct InMemoryStateInner {
    balances: HashMap<Address, u128>,
    nonces: HashMap<Address, u64>,
}

impl InMemoryState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InMemoryStateInner::default())),
        }
    }
}

#[async_trait]
impl State for InMemoryState {
    async fn get_balance(&self, addr: &Address) -> StateResult<u128> {
        let inner = self.inner.read().await;
        Ok(*inner.balances.get(addr).unwrap_or(&0u128))
    }

    async fn get_nonce(&self, addr: &Address) -> StateResult<u64> {
        let inner = self.inner.read().await;
        Ok(*inner.nonces.get(addr).unwrap_or(&0u64))
    }

    async fn apply_transfer(&self, sender: &Address, recipient: &Address, value: u128, expected_nonce: u64) -> StateResult<()> {
        let mut inner = self.inner.write().await;
        let current_nonce = *inner.nonces.get(sender).unwrap_or(&0u64);
        if current_nonce != expected_nonce {
            return Err(AccountStateError::NonceMismatch { expected: expected_nonce, found: current_nonce });
        }
        let sender_balance = *inner.balances.get(sender).unwrap_or(&0u128);
        if sender_balance < value {
            return Err(AccountStateError::InsufficientBalance);
        }
        // debit sender
        inner.balances.insert(sender.clone(), sender_balance - value);
        // credit recipient
        let rec_balance = *inner.balances.get(recipient).unwrap_or(&0u128);
        inner.balances.insert(recipient.clone(), rec_balance + value);
        // increment nonce
        inner.nonces.insert(sender.clone(), current_nonce + 1);
        Ok(())
    }

    async fn credit(&self, addr: &Address, value: u128) -> StateResult<()> {
        let mut inner = self.inner.write().await;
        let balance = *inner.balances.get(addr).unwrap_or(&0u128);
        inner.balances.insert(addr.clone(), balance + value);
        Ok(())
    }

    async fn set_nonce(&self, addr: &Address, nonce: u64) -> StateResult<()> {
        let mut inner = self.inner.write().await;
        inner.nonces.insert(addr.clone(), nonce);
        Ok(())
    }
}
