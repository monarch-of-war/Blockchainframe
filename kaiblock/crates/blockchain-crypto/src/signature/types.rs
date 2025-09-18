use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::{CryptoError, Result};

/// Ed25519 public key wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey(VerifyingKey);

/// Ed25519 private key wrapper  
#[derive(Clone)]
pub struct PrivateKey(SigningKey);

impl PublicKey {
    /// Create public key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKey(
                format!("Public key must be 32 bytes, got {}", bytes.len())
            ));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);
        
        VerifyingKey::from_bytes(&key_bytes)
            .map(PublicKey)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid public key: {}", e)))
    }
    
    /// Create public key from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(hex_str)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid hex: {}", e)))?;
        Self::from_bytes(&bytes)
    }
    
    /// Get the raw bytes of the public key
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
    
    /// Convert to hex string with 0x prefix
    pub fn to_hex_prefixed(&self) -> String {
        format!("0x{}", self.to_hex())
    }
    
    /// Verify a signature against a message
    pub fn verify(&self, message: &[u8], signature: &super::Signature) -> bool {
        let sig_bytes = signature.to_bytes();
        if let Ok(sig) = ed25519_dalek::Signature::from_bytes(&sig_bytes) {
            self.0.verify(message, &sig).is_ok()
        } else {
            false
        }
    }
}

impl PrivateKey {
    /// Create private key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKey(
                format!("Private key must be 32 bytes, got {}", bytes.len())
            ));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);
        
        Ok(PrivateKey(SigningKey::from_bytes(&key_bytes)))
    }
    
    /// Create private key from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(hex_str)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid hex: {}", e)))?;
        Self::from_bytes(&bytes)
    }
    
    /// Get the raw bytes of the private key
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
    
    /// Convert to hex string with 0x prefix
    pub fn to_hex_prefixed(&self) -> String {
        format!("0x{}", self.to_hex())
    }
    
    /// Get the corresponding public key
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }
    
    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> super::Signature {
        let signature = self.0.sign(message);
        super::Signature::from_bytes(signature.to_bytes())
    }
}

// Implement Debug for PrivateKey without exposing the key material
impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrivateKey([HIDDEN])")
    }
}

// Implement PartialEq for PrivateKey
impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<VerifyingKey> for PublicKey {
    fn from(key: VerifyingKey) -> Self {
        PublicKey(key)
    }
}

impl From<SigningKey> for PrivateKey {
    fn from(key: SigningKey) -> Self {
        PrivateKey(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_serialization() {
        let keypair = crate::signature::generate_keypair();
        let public_key = keypair.public_key();
        
        let hex = public_key.to_hex();
        let restored = PublicKey::from_hex(&hex).unwrap();
        
        assert_eq!(*public_key, restored);
    }

    #[test]
    fn test_private_key_serialization() {
        let keypair = crate::signature::generate_keypair();
        let private_key = keypair.private_key();
        
        let hex = private_key.to_hex();
        let restored = PrivateKey::from_hex(&hex).unwrap();
        
        assert_eq!(*private_key, restored);
    }

    #[test]
    fn test_public_key_from_private() {
        let keypair = crate::signature::generate_keypair();
        let private_key = keypair.private_key();
        let public_key1 = keypair.public_key();
        let public_key2 = private_key.public_key();
        
        assert_eq!(*public_key1, public_key2);
    }

    #[test]
    fn test_invalid_key_bytes() {
        // Test with wrong length
        let result = PublicKey::from_bytes(&[0u8; 16]);
        assert!(result.is_err());
        
        let result = PrivateKey::from_bytes(&[0u8; 16]);
        assert!(result.is_err());
    }
}