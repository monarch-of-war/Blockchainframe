// blockchain-core/src/ledger/block.rs
//! Core block and header definitions for the blockchain framework.
//!
//! This module defines:
//! - BlockHeader: canonical fields used across consensus engines
//! - Block: full block with transactions and consensus proof blob
//! - Helper methods for header hashing and merkle root calculation
//!
//! Design notes:
//! * `consensus_data` is opaque to the ledger — its meaning depends on the
//!   consensus engine (e.g., PoW nonce, PoS signatures, BFT votes).
//! * `header_bytes` serializes the header only, ensuring canonical
//!   consensus input across engines.

use serde::{Serialize, Deserialize};
use crate::ledger::transaction::TransactionTrait;
use blockchain_crypto::hash::sha256;

/// Canonical block header used across consensus engines.
/// This excludes consensus-specific proof fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    /// Hash of the parent block.
    pub prev_hash: Vec<u8>,
    /// Merkle root of the transactions in this block.
    pub merkle_root: Vec<u8>,
    /// UNIX timestamp in seconds.
    pub timestamp: u64,
    /// Protocol version number for upgrade handling.
    pub version: u32,
}

impl BlockHeader {
    /// Serialize the header deterministically for hashing/signing.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("header serialization must succeed")
    }

    /// Compute the header hash (SHA-256 double hash).
    pub fn hash(&self) -> Vec<u8> {
        let first = sha256(&self.to_bytes());
        sha256(&first)
    }
}

/// A block contains a header, transactions, and consensus proof data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block<T: TransactionTrait> {
    /// Canonical header (prev hash, merkle root, timestamp, version).
    pub header: BlockHeader,
    /// List of transactions included in this block.
    pub transactions: Vec<T>,
    /// Consensus-specific data (PoW nonce, PoS signatures, BFT votes, etc.).
    pub consensus_data: Vec<u8>,
}

impl<T: TransactionTrait> Block<T> {
    /// Return the canonical header bytes (without consensus_data).
    pub fn header_bytes(&self) -> Vec<u8> {
        self.header.to_bytes()
    }

    /// Return the block hash (double SHA-256 of header).
    pub fn hash(&self) -> Vec<u8> {
        self.header.hash()
    }

    /// Verify the block’s merkle root matches its transactions.
    pub fn verify_merkle_root(&self) -> bool {
        self.header.merkle_root == merkle_root(&self.transactions)
    }
}

/// Compute merkle root of transactions (generic over TransactionTrait).
/// For empty list, returns sha256 of empty bytes.
pub fn merkle_root<T: TransactionTrait>(txs: &[T]) -> Vec<u8> {
    if txs.is_empty() {
        return sha256(&[]);
    }

    // collect transaction hashes
    let mut hashes: Vec<Vec<u8>> = txs.iter().map(|tx| tx.hash()).collect();

    while hashes.len() > 1 {
        let mut next = Vec::with_capacity((hashes.len() + 1) / 2);
        for pair in hashes.chunks(2) {
            if pair.len() == 2 {
                let mut data = Vec::with_capacity(pair[0].len() + pair[1].len());
                data.extend_from_slice(&pair[0]);
                data.extend_from_slice(&pair[1]);
                next.push(sha256(&data));
            } else {
                // odd number, hash last element with itself
                let mut data = pair[0].clone();
                data.extend_from_slice(&pair[0]);
                next.push(sha256(&data));
            }
        }
        hashes = next;
    }

    hashes[0].clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::transaction::MockTransaction; // assumes you have a mock impl for testing

    #[test]
    fn test_merkle_root_empty() {
        let root = merkle_root::<MockTransaction>(&[]);
        assert_eq!(root, sha256(&[]));
    }

    #[test]
    fn test_header_hash_consistency() {
        let header = BlockHeader {
            prev_hash: vec![0u8; 32],
            merkle_root: sha256(b"txs"),
            timestamp: 42,
            version: 1,
        };
        let h1 = header.hash();
        let h2 = header.hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_block_merkle_verification() {
        let txs = vec![
            MockTransaction::new("a"),
            MockTransaction::new("b"),
        ];
        let root = merkle_root(&txs);
        let header = BlockHeader {
            prev_hash: vec![0u8; 32],
            merkle_root: root.clone(),
            timestamp: 42,
            version: 1,
        };
        let block = Block {
            header,
            transactions: txs,
            consensus_data: vec![],
        };
        assert!(block.verify_merkle_root());
    }
}
