use super::types::{Address, AddressType, AddressError};
use super::encoding::{Base58Encoder, HexEncoder};

/// Address validation utilities
pub struct AddressValidator;

impl AddressValidator {
    /// Validate an address structure
    pub fn is_valid(address: &Address) -> bool {
        // Check for zero address
        if address.hash160() == &[0u8; 20] {
            return false;
        }

        // Check address type is supported
        Self::is_supported_address_type(address.address_type())
    }

    /// Check if address type is supported
    pub fn is_supported_address_type(address_type: AddressType) -> bool {
        matches!(
            address_type,
            AddressType::P2PKH 
            | AddressType::P2SH 
            | AddressType::TestnetP2PKH 
            | AddressType::TestnetP2SH
        )
    }

    /// Parse address from string (tries multiple formats)
    pub fn parse_address(s: &str) -> Result<Address, AddressError> {
        // Try Base58 first (most common)
        if let Ok(address) = Base58Encoder::decode(s) {
            return Ok(address);
        }

        // Try hex format
        if let Ok(address) = HexEncoder::decode(s) {
            return Ok(address);
        }

        Err(AddressError::InvalidFormat {
            reason: "Address format not recognized".to_string(),
        })
    }

    /// Validate address string without parsing
    pub fn is_valid_address_string(s: &str) -> bool {
        Self::parse_address(s).is_ok()
    }

    /// Validate that an address matches expected network
    pub fn validate_network(address: &Address, mainnet: bool) -> bool {
        if mainnet {
            address.is_mainnet()
        } else {
            address.is_testnet()
        }
    }

    /// Comprehensive validation
    pub fn validate_comprehensive(
        address: &Address,
        require_mainnet: Option<bool>
    ) -> Result<(), AddressError> {
        // Basic validation
        if !Self::is_valid(address) {
            return Err(AddressError::InvalidFormat {
                reason: "Address failed basic validation".to_string(),
            });
        }

        // Network validation
        if let Some(mainnet) = require_mainnet {
            if !Self::validate_network(address, mainnet) {
                let network = if mainnet { "mainnet" } else { "testnet" };
                return Err(AddressError::InvalidFormat {
                    reason: format!("Address is not a {} address", network),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::conversion::AddressGenerator;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_valid_address() {
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let address = AddressGenerator::from_public_key(&keypair.public);

        assert!(AddressValidator::is_valid(&address));
    }

    #[test]
    fn test_zero_address_invalid() {
        let zero_address = Address::new([0u8; 20], AddressType::P2PKH);
        assert!(!AddressValidator::is_valid(&zero_address));
    }

    #[test]
    fn test_network_validation() {
        let hash160 = [1u8; 20];
        let mainnet_addr = Address::new(hash160, AddressType::P2PKH);
        let testnet_addr = Address::new(hash160, AddressType::TestnetP2PKH);

        assert!(AddressValidator::validate_network(&mainnet_addr, true));
        assert!(!AddressValidator::validate_network(&mainnet_addr, false));
        assert!(AddressValidator::validate_network(&testnet_addr, false));
        assert!(!AddressValidator::validate_network(&testnet_addr, true));
    }

    #[test]
    fn test_comprehensive_validation() {
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let address = AddressGenerator::from_public_key(&keypair.public);

        // Should pass comprehensive validation
        assert!(AddressValidator::validate_comprehensive(&address, Some(true)).is_ok());
        assert!(AddressValidator::validate_comprehensive(&address, None).is_ok());

        // Should fail network validation
        assert!(AddressValidator::validate_comprehensive(&address, Some(false)).is_err());
    }
}