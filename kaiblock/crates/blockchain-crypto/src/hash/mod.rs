mod merkle;
mod types;
mod utils;

pub use merkle::{MerkleTree, MerkleProof};
pub use types::Hash256;
pub use utils::*;

use sha2::{Sha256, Digest};
use crate::{CryptoError, Result};

/// SHA-256 hash function wrapper
pub fn sha256(data: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    Hash256::from_bytes(hasher.finalize().into())
}

/// Double SHA-256 hash (commonly used in Bitcoin)
pub fn double_sha256(data: &[u8]) -> Hash256 {
    let first_hash = sha256(data);
    sha256(first_hash.as_bytes())
}

/// Hash multiple data pieces together
pub fn hash_combine(data: &[&[u8]]) -> Hash256 {
    let mut hasher = Sha256::new();
    for chunk in data {
        hasher.update(chunk);
    }
    Hash256::from_bytes(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = sha256(data);
        
        // Known SHA-256 hash of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash.to_hex(), expected);
    }

    #[test]
    fn test_double_sha256() {
        let data = b"test";
        let hash = double_sha256(data);
        
        // Should be different from single hash
        let single_hash = sha256(data);
        assert_ne!(hash, single_hash);
    }

    #[test]
    fn test_hash_combine() {
        let data1 = b"hello";
        let data2 = b"world";
        
        let combined = hash_combine(&[data1, data2]);
        let direct = sha256(b"helloworld");
        
        assert_eq!(combined, direct);
    }
}