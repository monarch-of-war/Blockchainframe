// validation/pipeline.rs
//! Validation pipeline for transactions used by consensus validate_and_commit.
//!
//! This pipeline is intentionally minimal and pluggable.
//! - Runs signature verification (tx.verify())
//! - Checks nonce and balance via the State trait (account model)
//! - Applies the transaction to the State (state transition)
//!
//! The pipeline is generic over `Tx` which must implement `AccountTransaction` defined below.

use crate::consensus::ConsensusError;
use crate::state::{State, Address};
use async_trait::async_trait;
use std::sync::Arc;

/// Transaction type required by this default validation pipeline.
/// This is an account-model transaction interface. If you use UTXO, implement a different pipeline.
pub trait AccountTransaction: Send + Sync {
    /// Binary hash / id of the transaction (used in mempool & merkle)
    fn hash(&self) -> Vec<u8>;

    /// Verify cryptographic signature(s) of the transaction (returns true on success).
    fn verify(&self) -> bool;

    /// Sender address bytes
    fn sender(&self) -> Address;

    /// Recipient address bytes
    fn recipient(&self) -> Address;

    /// Transfer value (u128)
    fn value(&self) -> u128;

    /// Expected nonce for the sender
    fn nonce(&self) -> u64;
}

/// Result type returned by validation pipeline steps.
pub type ValidationResult<T> = Result<T, ConsensusError>;

#[async_trait]
pub trait ValidationPipeline<Tx: AccountTransaction + 'static>: Send + Sync {
    /// Validate a single transaction (signature + state checks), and apply to state if valid.
    async fn validate_and_apply(&self, tx: &Tx) -> ValidationResult<()>;
}

/// Default validation pipeline implementation for the account-model.
/// Uses a shared `State` instance (e.g., InMemoryState or DB adapter).
pub struct DefaultValidationPipeline {
    pub state: Arc<dyn State>,
}

impl DefaultValidationPipeline {
    pub fn new(state: Arc<dyn State>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl<Tx> ValidationPipeline<Tx> for DefaultValidationPipeline
where
    Tx: AccountTransaction + 'static,
{
    async fn validate_and_apply(&self, tx: &Tx) -> ValidationResult<()> {
        // 1) signature verification
        if !tx.verify() {
            return Err(ConsensusError::InvalidBlock("tx signature invalid".into()));
        }

        // 2) check nonce and balance via state
        let sender = tx.sender();
        let recipient = tx.recipient();
        let expected_nonce = tx.nonce();
        let value = tx.value();

        // get on-chain nonce and balance
        let onchain_nonce = self.state.get_nonce(&sender).await
            .map_err(|e| ConsensusError::Internal(format!("state error: {:?}", e)))?;
        if onchain_nonce != expected_nonce {
            return Err(ConsensusError::InvalidBlock(format!("nonce mismatch: expected {}, got {}", expected_nonce, onchain_nonce)));
        }

        let balance = self.state.get_balance(&sender).await
            .map_err(|e| ConsensusError::Internal(format!("state error: {:?}", e)))?;
        if balance < value {
            return Err(ConsensusError::InvalidBlock("insufficient balance".into()));
        }

        // 3) apply the transfer to state (debit/credit + increment nonce)
        self.state.apply_transfer(&sender, &recipient, value, expected_nonce).await
            .map_err(|e| ConsensusError::InvalidBlock(format!("state apply failed: {:?}", e)))?;

        Ok(())
    }
}
