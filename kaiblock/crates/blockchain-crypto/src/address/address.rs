use super::AddressType;
use crate::signature::PublicKey;
use crate::hash::{sha256, Hash256};
use crate::{CryptoError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Blockchain address derived from public key
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    address_type: AddressType,
    data: Vec<u8>,
    encoded: String,
}

impl Address {
    /// Create address from public key
    pub fn from_public_key(public_key: &PublicKey, address_type: AddressType) -> Self {
        let public_key_bytes = public_key.to_bytes();
        let hash = sha256(&public_key_bytes);
        
        match address_type {
            AddressType::Base58 => Self::create_base58_address(hash),
            AddressType::HexChecksum => Self::create_hex_checksum_address(hash),
            AddressType::Hex => Self::create_hex_address(hash),
        }
    }
    
    /// Create address from hash and type
    pub fn from_hash(hash: Hash256, address_type: AddressType) -> Self {
        match address_type {
            AddressType::Base58 => Self::create_base58_address(hash),
            AddressType::HexChecksum => Self::create_hex_checksum_address(hash),
            AddressType::Hex => Self::create_hex_address(hash),
        }
    }
    
    /// Parse address from string
    pub fn from_string(address_str: &str) -> Result<Self> {
        let address_type = AddressType::detect(address_str)
            .ok_or_else(|| CryptoError::AddressError("Unknown address format".to_string()))?;
        
        match address_type {
            AddressType::Base58 => Self::parse_base58_address(address_str),
            AddressType::HexChecksum => Self::parse_hex_checksum_address(address_str),
            AddressType::Hex => Self::parse_hex_address(address_str),
        }
    }
    
    /// Get address type
    pub fn address_type(&self) -> AddressType {
        self.address_type
    }
    
    /// Get raw address data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Get encoded address string
    pub fn encoded(&self) -> &str {
        &self.encoded
    }
    
    /// Validate an address string
    pub fn validate(address_str: &str) -> Result<AddressType> {
        let address = Self::from_string(address_str)?;
        Ok(address.address_type)
    }
    
    /// Create Base58 address (Bitcoin-style)
    fn create_base58_address(hash: Hash256) -> Self {
        // Take first 20 bytes of hash
        let mut data = Vec::with_capacity(21);
        data.push(0x00); // Version byte for mainnet
        data.extend_from_slice(&hash.as_bytes()[..20]);
        
        // Add checksum
        let checksum_hash = sha256(&sha256(&data).as_bytes());
        data.extend_from_slice(&checksum_hash.as_bytes()[..4]);
        
        let encoded = bs58::encode(&data).into_string();
        
        Self {
            address_type: AddressType::Base58,
            data: data[1..21].to_vec(), // Store without version and checksum
            encoded,
        }
    }
    
    /// Create hex address with checksum (Ethereum-style)
    fn create_hex_checksum_address(hash: Hash256) -> Self {
        // Take last 20 bytes of hash (Ethereum convention)
        let data = &hash.as_bytes()[12..];
        let hex_str = hex::encode(data);
        
        // Apply checksum (simplified version)
        let checksum_hash = sha256(hex_str.as_bytes());
        let mut checksummed = String::with_capacity(42);
        checksummed.push_str("0x");
        
        for (i, c) in hex_str.chars().enumerate() {
            if c.is_ascii_alphabetic() {
                if checksum_hash.as_bytes()[i / 2] & (if i % 2 == 0 { 0x80 } else { 0x08 }) != 0 {
                    checksummed.push(c.to_ascii_uppercase());
                } else {
                    checksummed.push(c.to_ascii_lowercase());
                }
            } else {
                checksummed.push(c);
            }
        }
        
        Self {
            address_type: AddressType::HexChecksum,
            data: data.to_vec(),
            encoded: checksummed,
        }
    }
    
    /// Create simple hex address
    fn create_hex_address(hash: Hash256) -> Self {
        let data = &hash.as_bytes()[12..]; // Take last 20 bytes
        let encoded = format!("0x{}", hex::encode(data));
        
        Self {
            address_type: AddressType::Hex,
            data: data.to_vec(),
            encoded,
        }
    }
    
    /// Parse Base58 address
    fn parse_base58_address(address_str: &str) -> Result<Self> {
        let decoded = bs58::decode(address_str)
            .into_vec()
            .map_err(|e| CryptoError::AddressError(format!("Invalid Base58: {}", e)))?;
        
        if decoded.len() != 25 {
            return Err(CryptoError::AddressError("Invalid Base58 address length".to_string()));
        }
        
        // Verify checksum
        let payload = &decoded[..21];
        let checksum = &decoded[21..];
        let expected_checksum = &sha256(&sha256(payload).as_bytes()).as_bytes()[..4];
        
        if checksum != expected_checksum {
            return Err(CryptoError::AddressError("Invalid checksum".to_string()));
        }
        
        Ok(Self {
            address_type: AddressType::Base58,
            data: payload[1..].to_vec(), // Remove version byte
            encoded: address_str.to_string(),
        })
    }
    
    /// Parse hex checksum address
    fn parse_hex_checksum_address(address_str: &str) -> Result<Self> {
        if address_str.len() != 42 {
            return Err(CryptoError::AddressError("Invalid hex address length".to_string()));
        }
        
        let hex_part = &address_str[2..];
        let data = hex::decode(hex_part)
            .map_err(|e| CryptoError::AddressError(format!("Invalid hex: {}", e)))?;
        
        // Verify checksum (simplified)
        let lowercase_hex = hex_part.to_lowercase();
        let checksum_hash = sha256(lowercase_hex.as_bytes());
        
        for (i, c) in hex_part.chars().enumerate() {
            if c.is_ascii_alphabetic() {
                let should_be_upper = checksum_hash.as_bytes()[i / 2] & (if i % 2 == 0 { 0x80 } else { 0x08 }) != 0;
                if should_be_upper != c.is_ascii_uppercase() {
                    return Err(CryptoError::AddressError("Invalid checksum".to_string()));
                }
            }
        }
        
        Ok(Self {
            address_type: AddressType::HexChecksum,
            data,
            encoded: address_str.to_string(),
        })
    }
    
    /// Parse simple hex address
    fn parse_hex_address(address_str: &str) -> Result<Self> {
        if !address_str.starts_with("0x") {
            return Err(CryptoError::AddressError("Hex address must start with 0x".to_string()));
        }
        
        let hex_part = &address_str[2..];
        let data = hex::decode(hex_part)
            .map_err(|e| CryptoError::AddressError(format!("Invalid hex: {}", e)))?;
        
        Ok(Self {
            address_type: AddressType::Hex,
            data,
            encoded: address_str.to_string(),
        })
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encoded)
    }
}

impl From<Address> for String {
    fn from(address: Address) -> Self {
        address.encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::generate_keypair;

    #[test]
    fn test_base58_address_creation() {
        let keypair = generate_keypair();
        let address = Address::from_public_key(keypair.public_key(), AddressType::Base58);
        
        assert_eq!(address.address_type(), AddressType::Base58);
        assert!(!address.encoded().is_empty());
        assert!(address.encoded().starts_with('1') || address.encoded().starts_with('3'));
    }

    #[test]
    fn test_hex_checksum_address_creation() {
        let keypair = generate_keypair();
        let address = Address::from_public_key(keypair.public_key(), AddressType::HexChecksum);
        
        assert_eq!(address.address_type(), AddressType::HexChecksum);
        assert_eq!(address.encoded().len(), 42);
        assert!(address.encoded().starts_with("0x"));
    }

    #[test]
    fn test_address_roundtrip() {
        let keypair = generate_keypair();
        
        // Test Base58 roundtrip
        let address1 = Address::from_public_key(keypair.public_key(), AddressType::Base58);
        let parsed1 = Address::from_string(&address1.encoded()).unwrap();
        assert_eq!(address1.data(), parsed1.data());
        assert_eq!(address1.address_type(), parsed1.address_type());
        
        // Test HexChecksum roundtrip
        let address2 = Address::from_public_key(keypair.public_key(), AddressType::HexChecksum);
        let parsed2 = Address::from_string(&address2.encoded()).unwrap();
        assert_eq!(address2.data(), parsed2.data());
        assert_eq!(address2.address_type(), parsed2.address_type());
        
        // Test Hex roundtrip
        let address3 = Address::from_public_key(keypair.public_key(), AddressType::Hex);
        let parsed3 = Address::from_string(&address3.encoded()).unwrap();
        assert_eq!(address3.data(), parsed3.data());
        assert_eq!(address3.address_type(), parsed3.address_type());
    }

    #[test]
    fn test_address_validation() {
        let keypair = generate_keypair();
        let address = Address::from_public_key(keypair.public_key(), AddressType::Base58);
        
        // Valid address should validate
        assert!(Address::validate(&address.encoded()).is_ok());
        
        // Invalid addresses should fail
        assert!(Address::validate("invalid").is_err());
        assert!(Address::validate("").is_err());
        assert!(Address::validate("1234567890").is_err());
    }

    #[test]
    fn test_different_address_types_same_key() {
        let keypair = generate_keypair();
        
        let base58_addr = Address::from_public_key(keypair.public_key(), AddressType::Base58);
        let hex_addr = Address::from_public_key(keypair.public_key(), AddressType::HexChecksum);
        
        // Different formats should produce different encoded strings
        assert_ne!(base58_addr.encoded(), hex_addr.encoded());
        
        // But both should be valid
        assert!(Address::validate(base58_addr.encoded()).is_ok());
        assert!(Address::validate(hex_addr.encoded()).is_ok());
    }

    #[test]
    fn test_address_display() {
        let keypair = generate_keypair();
        let address = Address::from_public_key(keypair.public_key(), AddressType::Base58);
        
        let display_str = format!("{}", address);
        assert_eq!(display_str, address.encoded());
    }

    #[test]
    fn test_corrupted_base58_checksum() {
        let keypair = generate_keypair();
        let mut address = Address::from_public_key(keypair.public_key(), AddressType::Base58);
        
        // Corrupt the last character (checksum)
        let mut corrupted = address.encoded().chars().collect::<Vec<_>>();
        let last_idx = corrupted.len() - 1;
        corrupted[last_idx] = if corrupted[last_idx] == '1' { '2' } else { '1' };
        let corrupted_str: String = corrupted.into_iter().collect();
        
        // Should fail validation
        assert!(Address::from_string(&corrupted_str).is_err());
    }
}