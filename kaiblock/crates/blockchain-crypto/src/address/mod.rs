//! Address generation and management for the blockchain
//! 
//! This module provides functionality for:
//! - Converting public keys to addresses using conversion.rs
//! - Address validation and verification using validation.rs
//! - Multiple encoding formats (Base58, Hex) using encoding.rs
//! - Checksum validation for error detection using validation.rs

pub mod conversion;
pub mod validation;
pub mod encoding;
pub mod types;

// Re-export the main types and functions for easy access
pub use types::{Address, AddressType, AddressError};
pub use conversion::AddressGenerator;
pub use validation::AddressValidator;
pub use encoding::{Base58Encoder, HexEncoder};

// Convenient type aliases
pub type Result<T> = std::result::Result<T, AddressError>;

// Public API convenience functions
pub fn generate_address(public_key: &ed25519_dalek::PublicKey) -> Address {
    AddressGenerator::from_public_key(public_key)
}

pub fn validate_address(address: &Address) -> bool {
    AddressValidator::is_valid(address)
}

pub fn address_from_string(s: &str) -> Result<Address> {
    AddressValidator::parse_address(s)
}