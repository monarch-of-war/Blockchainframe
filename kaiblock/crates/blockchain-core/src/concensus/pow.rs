//! Proof-of-Work consensus implementation (Bitcoin-style).
//!
//! This module provides mining and verification utilities
//! based on a target threshold encoded in compact form (`nBits`).

use crate::consensus::{Consensus, difficulty::{compact_to_target, retarget}};
use num_bigint::BigUint;
use sha2::{Digest, Sha256};

/// Bitcoin-style defaults (can be overridden by chain config).
pub const BITCOIN_INIT_NBITS: u32 = 0x1d00ffff; // Difficulty 1 (genesis)
pub const RETARGET_INTERVAL: u64 = 2016;        // Blocks per adjustment
pub const TARGET_BLOCK_TIME: u64 = 600;         // Seconds (10 minutes)

/// Possible errors during PoW verification.
#[derive(Debug, PartialEq, Eq)]
pub enum PowError {
    InvalidNonceLength,
    HashAboveTarget,
}

/// Proof-of-Work consensus engine.
pub struct ProofOfWork {
    pub nbits: u32,      // Difficulty target in compact format
    pub max_nonce: u64,  // Search space for mining
}

impl ProofOfWork {
    /// Double SHA256 (Bitcoin PoW hash).
    fn double_sha256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let first_hash = hasher.finalize_reset();

        hasher.update(&first_hash);
        hasher.finalize().to_vec()
    }

    /// Computes the PoW hash for a given header + nonce.
    pub fn pow_hash(block_header: &[u8], nonce: u64) -> Vec<u8> {
        let mut data = Vec::with_capacity(block_header.len() + 8);
        data.extend_from_slice(block_header);
        data.extend_from_slice(&nonce.to_le_bytes());
        Self::double_sha256(&data)
    }

    /// Adjusts difficulty using Bitcoin-style retarget formula.
    pub fn adjust_difficulty(
        &self,
        old_target: &BigUint,
        actual_time: u64,
        expected_time: u64,
    ) -> BigUint {
        retarget(old_target, actual_time, expected_time)
    }
}

impl Consensus for ProofOfWork {
    fn verify(&self, block_header: &[u8], consensus_data: &[u8]) -> bool {
        if consensus_data.len() != 8 {
            return false; // or Err(PowError::InvalidNonceLength) in a Result-based design
        }
        let nonce = u64::from_le_bytes(consensus_data.try_into().unwrap());
        let target = compact_to_target(self.nbits);

        let final_hash = Self::pow_hash(block_header, nonce);
        let hash_int = BigUint::from_bytes_be(&final_hash);

        hash_int <= target
    }

    fn produce(&self, block_header: &[u8]) -> Option<Vec<u8>> {
        let target = compact_to_target(self.nbits);

        for nonce in 0..self.max_nonce {
            let final_hash = Self::pow_hash(block_header, nonce);
            let hash_int = BigUint::from_bytes_be(&final_hash);

            if hash_int <= target {
                return Some(nonce.to_le_bytes().to_vec());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::difficulty::{target_to_compact};

    #[test]
    fn test_pow_hash_consistency() {
        let header = b"test-header";
        let nonce = 42;
        let h1 = ProofOfWork::pow_hash(header, nonce);
        let h2 = ProofOfWork::pow_hash(header, nonce);
        assert_eq!(h1, h2, "Hash must be deterministic");
    }

    #[test]
    fn test_mining_with_easy_difficulty() {
        let pow = ProofOfWork {
            nbits: 0x207fffff, // Very easy target
            max_nonce: 100_000,
        };

        let header = b"block-header";
        let result = pow.produce(header);
        assert!(result.is_some(), "Should mine block under low difficulty");
    }

    #[test]
    fn test_verify_pow_success_and_failure() {
        let pow = ProofOfWork {
            nbits: 0x207fffff,
            max_nonce: 10_000,
        };

        let header = b"verify-header";
        if let Some(consensus_data) = pow.produce(header) {
            assert!(pow.verify(header, &consensus_data), "Valid nonce must pass verification");

            let bad_nonce = (u64::MAX).to_le_bytes().to_vec();
            assert!(!pow.verify(header, &bad_nonce), "Invalid nonce must fail verification");
        } else {
            panic!("Mining failed unexpectedly");
        }
    }

    #[test]
    fn test_adjust_difficulty_up_and_down() {
        let pow = ProofOfWork {
            nbits: BITCOIN_INIT_NBITS,
            max_nonce: 1_000,
        };

        let old_target = compact_to_target(BITCOIN_INIT_NBITS);

        // Too fast → difficulty must go up (target smaller)
        let faster = pow.adjust_difficulty(&old_target, TARGET_BLOCK_TIME / 2, TARGET_BLOCK_TIME);
        assert!(faster < old_target);

        // Too slow → difficulty must go down (target larger)
        let slower = pow.adjust_difficulty(&old_target, TARGET_BLOCK_TIME * 2, TARGET_BLOCK_TIME);
        assert!(slower > old_target);
    }
}
