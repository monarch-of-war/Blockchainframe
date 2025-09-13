// blockchain-core/src/ledger/merkle.rs

use blockchain_crypto::hash::sha256;
use super::transaction::TransactionTrait;

/// Compute merkle root from transactions
pub fn merkle_root<T: TransactionTrait>(transactions: &[T]) -> Vec<u8> {
    if transactions.is_empty() {
        return sha256(&[]);
    }

    let mut hashes: Vec<Vec<u8>> = transactions.iter().map(|tx| tx.txid()).collect();

    while hashes.len() > 1 {
        let mut new_hashes = Vec::new();
        for chunk in hashes.chunks(2) {
            if chunk.len() == 2 {
                let mut combined = chunk[0].clone();
                combined.extend_from_slice(&chunk[1]);
                new_hashes.push(sha256(&combined));
            } else {
                new_hashes.push(chunk[0].clone()); // odd number, carry forward
            }
        }
        hashes = new_hashes;
    }

    hashes[0].clone()
}
