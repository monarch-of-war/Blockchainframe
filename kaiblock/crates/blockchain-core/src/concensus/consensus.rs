// src/consensus/consensus.rs

/// Consensus trait defines the common interface for all consensus mechanisms.
/// This allows developers to implement PoW, PoS, PBFT, or custom algorithms.
pub trait Consensus {
    /// Runs the consensus step to reach agreement on the next block.
    /// Returns `true` if consensus was reached and block is accepted.
    fn validate_block(&self, block_data: &[u8]) -> bool;

    /// Returns the name of the consensus mechanism (e.g., "PoW", "PoS").
    fn algorithm_name(&self) -> &'static str;

    /// Optional: Allows the consensus to update internal state after block finalization.
    fn finalize_block(&mut self, block_data: &[u8]);
}

/// A simple placeholder implementation for demonstration.
pub struct DummyConsensus;

impl Consensus for DummyConsensus {
    fn validate_block(&self, _block_data: &[u8]) -> bool {
        // For now, always returns true (accepts all blocks).
        true
    }

    fn algorithm_name(&self) -> &'static str {
        "DummyConsensus"
    }

    fn finalize_block(&mut self, _block_data: &[u8]) {
        // No-op for dummy.
    }
}
