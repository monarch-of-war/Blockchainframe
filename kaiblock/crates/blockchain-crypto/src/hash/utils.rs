use super::{Hash256, sha256};

/// Calculate hash of serialized data
pub fn hash_serialize<T: serde::Serialize>(data: &T) -> crate::Result<Hash256> {
    let serialized = serde_json::to_vec(data)
        .map_err(|e| crate::CryptoError::SerializationError(e.to_string()))?;
    Ok(sha256(&serialized))
}

/// Create a hash chain (each hash depends on the previous)
pub fn hash_chain(data: &[&[u8]]) -> Vec<Hash256> {
    let mut hashes = Vec::new();
    let mut previous = Hash256::zero();
    
    for chunk in data {
        let combined = crate::hash::hash_combine(&[previous.as_bytes(), chunk]);
        hashes.push(combined);
        previous = combined;
    }
    
    hashes
}

/// Generate a random hash (for testing purposes)
#[cfg(test)]
pub fn random_hash() -> Hash256 {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    Hash256::from_bytes(bytes)
}

/// Calculate the difficulty of a hash (number of leading zero bits)
pub fn hash_difficulty(hash: &Hash256) -> u32 {
    let bytes = hash.as_bytes();
    let mut difficulty = 0;
    
    for &byte in bytes {
        if byte == 0 {
            difficulty += 8;
        } else {
            difficulty += byte.leading_zeros();
            break;
        }
    }
    
    difficulty
}

/// Check if a hash meets a minimum difficulty requirement
pub fn meets_difficulty(hash: &Hash256, target_difficulty: u32) -> bool {
    hash_difficulty(hash) >= target_difficulty
}

/// Create a target hash for a given difficulty
pub fn difficulty_target(difficulty: u32) -> Hash256 {
    let mut bytes = [0xffu8; 32];
    
    let full_bytes = difficulty / 8;
    let remaining_bits = difficulty % 8;
    
    // Set full bytes to zero
    for i in 0..full_bytes as usize {
        if i < 32 {
            bytes[i] = 0;
        }
    }
    
    // Set remaining bits in the next byte
    if full_bytes < 32 && remaining_bits > 0 {
        let mask = 0xff >> remaining_bits;
        bytes[full_bytes as usize] &= mask;
    }
    
    Hash256::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestData {
        value: u64,
        name: String,
    }

    #[test]
    fn test_hash_serialize() {
        let data = TestData {
            value: 42,
            name: "test".to_string(),
        };
        
        let hash1 = hash_serialize(&data).unwrap();
        let hash2 = hash_serialize(&data).unwrap();
        
        // Same data should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different data should produce different hash
        let data2 = TestData {
            value: 43,
            name: "test".to_string(),
        };
        let hash3 = hash_serialize(&data2).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_chain() {
        let data = vec![b"block1", b"block2", b"block3"];
        let chain = hash_chain(&data);
        
        assert_eq!(chain.len(), 3);
        
        // Each hash should be different
        assert_ne!(chain[0], chain[1]);
        assert_ne!(chain[1], chain[2]);
        
        // Chain should be deterministic
        let chain2 = hash_chain(&data);
        assert_eq!(chain, chain2);
    }

    #[test]
    fn test_hash_difficulty() {
        // Create a hash with known leading zeros
        let mut bytes = [0u8; 32];
        bytes[0] = 0x00; // 8 leading zeros
        bytes[1] = 0x80; // 0 additional leading zeros (starts with 1)
        
        let hash = Hash256::from_bytes(bytes);
        assert_eq!(hash_difficulty(&hash), 8);
        
        // Test with more leading zeros
        bytes[1] = 0x00; // 8 more zeros
        bytes[2] = 0x40; // 1 more zero
        let hash2 = Hash256::from_bytes(bytes);
        assert_eq!(hash_difficulty(&hash2), 17);
    }

    #[test]
    fn test_meets_difficulty() {
        let mut bytes = [0u8; 32];
        bytes[2] = 0x01; // 23 leading zeros
        let hash = Hash256::from_bytes(bytes);
        
        assert!(meets_difficulty(&hash, 20));
        assert!(meets_difficulty(&hash, 23));
        assert!(!meets_difficulty(&hash, 24));
    }

    #[test]
    fn test_difficulty_target() {
        let target = difficulty_target(16);
        
        // Should have at least 16 leading zero bits
        assert!(hash_difficulty(&target) >= 16);
        
        // Zero difficulty should give max target
        let max_target = difficulty_target(0);
        assert_eq!(max_target.as_bytes(), &[0xff; 32]);
    }
}