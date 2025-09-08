use serde::{Serialize, Deserialize};
use std::fmt;

/// Raw hash types
pub type Hash256 = [u8; 32];
pub type Hash160 = [u8; 20];

/// Address version bytes for different address types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    /// Pay-to-Public-Key-Hash (standard addresses)
    P2PKH = 0x00,
    /// Pay-to-Script-Hash (for future smart contracts)
    P2SH = 0x05,
    /// Testnet addresses
    TestnetP2PKH = 0x6F,
    TestnetP2SH = 0xC4,
}

/// Main address structure containing the hash and type information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    /// The 20-byte hash160 of the public key
    pub hash160: Hash160,
    /// Address type/version for encoding
    pub address_type: AddressType,
}

impl Address {
    /// Create a new address with the given hash and type
    pub fn new(hash160: Hash160, address_type: AddressType) -> Self {
        Self {
            hash160,
            address_type,
        }
    }

    /// Create a standard P2PKH address
    pub fn p2pkh(hash160: Hash160) -> Self {
        Self::new(hash160, AddressType::P2PKH)
    }

    /// Create a testnet P2PKH address
    pub fn testnet_p2pkh(hash160: Hash160) -> Self {
        Self::new(hash160, AddressType::TestnetP2PKH)
    }

    /// Get the raw hash160 bytes of the address 
    pub fn hash160(&self) -> &Hash160 {
        &self.hash160
    }

    /// Get the address type
    pub fn address_type(&self) -> AddressType {
        self.address_type
    }

    /// Get version byte for this address type
    pub fn version_byte(&self) -> u8 {
        self.address_type as u8
    }

    /// Check if this is a mainnet address
    pub fn is_mainnet(&self) -> bool {
        matches!(self.address_type, AddressType::P2PKH | AddressType::P2SH)
    }

    /// Check if this is a testnet address
    pub fn is_testnet(&self) -> bool {
        matches!(self.address_type, AddressType::TestnetP2PKH | AddressType::TestnetP2SH)
    }

    /// Convert to bytes for hashing/serialization
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(21);
        bytes.push(self.version_byte());
        bytes.extend_from_slice(&self.hash160);
        bytes
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Default to Base58 encoding for display
        match crate::address::encoding::Base58Encoder::encode(self) {
            Ok(encoded) => write!(f, "{}", encoded),
            Err(_) => write!(f, "Invalid Address"),
        }
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.hash160
    }
}

/// Errors that can occur during address operations
#[derive(Debug, thiserror::Error)]
pub enum AddressError {
    #[error("Invalid address length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Invalid address format: {reason}")]
    InvalidFormat { reason: String },

    #[error("Invalid checksum in address")]
    InvalidChecksum,

    #[error("Unsupported address type: {type_byte:#04x}")]
    UnsupportedAddressType { type_byte: u8 },

    #[error("Invalid encoding: {details}")]
    EncodingError { details: String },

    #[error("Address cannot be zero")]
    ZeroAddress,
}

/// Trait for types that can be converted to an Address
pub trait ToAddress {
    fn to_address(&self, address_type: AddressType) -> Address;
}

/// Trait for address encoding formats
pub trait AddressEncoder {
    fn encode(address: &Address) -> Result<String, AddressError>;
    fn decode(encoded: &str) -> Result<Address, AddressError>;
}