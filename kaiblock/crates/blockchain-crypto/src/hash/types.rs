use serde::{Deserialize, Serialize};
use std::fmt;
use crate::{CryptoError, Result};

/// 256-bit hash value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    /// Create a new hash from 32 bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    /// Create a hash from a byte slice
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 32 {
            return Err(CryptoError::InvalidHash(
                format!("Expected 32 bytes, got {}", slice.len())
            ));
        }
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }
    
    /// Create a hash from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        
        let bytes = hex::decode(hex_str)
            .map_err(|e| CryptoError::InvalidHash(format!("Invalid hex: {}", e)))?;
        
        Self::from_slice(&bytes)
    }
    
    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    /// Convert to byte slice
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
    
    /// Convert to hex string with 0x prefix
    pub fn to_hex_prefixed(&self) -> String {
        format!("0x{}", self.to_hex())
    }
    
    /// Create a zero hash
    pub fn zero() -> Self {
        Self([0u8; 32])
    }
    
    /// Check if hash is zero
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl Default for Hash256 {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for Hash256 {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Hash256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash256_creation() {
        let bytes = [1u8; 32];
        let hash = Hash256::from_bytes(bytes);
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_hex_conversion() {
        let hash = Hash256::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
                                       0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
                                       0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
                                       0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
        
        let hex = hash.to_hex();
        let restored = Hash256::from_hex(&hex).unwrap();
        assert_eq!(hash, restored);
    }

    #[test]
    fn test_zero_hash() {
        let zero = Hash256::zero();
        assert!(zero.is_zero());
        assert_eq!(zero.to_hex(), "0".repeat(64));
    }
}