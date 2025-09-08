// pure transformation logic (pubkey → hash → address).pure transformation logic (pubkey → hash → address).

use super::types::{Address, AddressType, Hash160, Hash256};
use sha2::{Sha256, Digest};
use ripemd::{Ripemd160, Digest as RipemdDigest};
use ed25519_dalek::PublicKey;

/// Address generation utilities
pub struct AddressGenerator;

impl AddressGenerator {
    /// Generate address from Ed25519 public key
    /// 
    /// Process:
    /// 1. Take public key bytes (32 bytes)
    /// 2. SHA-256 hash → 32 bytes  
    /// 3. RIPEMD-160 hash → 20 bytes
    /// 4. Create Address with specified type
    /// 
    /// # Arguments
    /// * `public_key` - Ed25519 public key to convert
    /// * `address_type` - Type of address to generate (defaults to P2PKH)
    /// 
    /// # Returns
    /// * `Address` containing the hash160 and type

    pub fn from_public_key(public_key: &PublicKey) -> Address {
        Self::from_public_key_with_type(public_key, AddressType::P2PKH)
    }

    /// Generate address with specific type
    pub fn from_public_key_with_type(
        public_key: &PublicKey, 
        address_type: AddressType
    ) -> Address {
        let hash160 = Self::public_key_to_hash160(public_key.as_bytes());
        Address::new(hash160, address_type)
    }

    /// Generate address from raw public key bytes
    pub fn from_public_key_bytes(pubkey_bytes: &[u8]) -> Address {
        Self::from_public_key_bytes_with_type(pubkey_bytes, AddressType::P2PKH)
    }

    /// Generate address from raw bytes with specific type
    pub fn from_public_key_bytes_with_type(
        pubkey_bytes: &[u8],
        address_type: AddressType
    ) -> Address {
        let hash160 = Self::public_key_to_hash160(pubkey_bytes);
        Address::new(hash160, address_type)
    }

    /// Core hashing function: Public Key → SHA-256 → RIPEMD-160
    fn public_key_to_hash160(pubkey_bytes: &[u8]) -> Hash160 {
        // Step 1: SHA-256 hash
        let mut sha256_hasher = Sha256::new();
        sha256_hasher.update(pubkey_bytes);
        let sha256_result: Hash256 = sha256_hasher.finalize().into();

        // Step 2: RIPEMD-160 hash  
        let mut ripemd160_hasher = Ripemd160::new();
        ripemd160_hasher.update(&sha256_result);
        let ripemd160_result: Hash160 = ripemd160_hasher.finalize().into();

        ripemd160_result
    }

    /// Verify that a public key generates the expected address
    pub fn verify_address(public_key: &PublicKey, expected_address: &Address) -> bool {
        let generated_address = Self::from_public_key_with_type(
            public_key, 
            expected_address.address_type()
        );
        generated_address == *expected_address
    }

    /// Generate multiple address types from one public key
    pub fn generate_all_types(public_key: &PublicKey) -> Vec<Address> {
        let hash160 = Self::public_key_to_hash160(public_key.as_bytes());
        
        vec![
            Address::new(hash160, AddressType::P2PKH),
            Address::new(hash160, AddressType::TestnetP2PKH),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_address_generation_deterministic() {
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);

        // Generate address twice
        let address1 = AddressGenerator::from_public_key(&keypair.public);
        let address2 = AddressGenerator::from_public_key(&keypair.public);

        // Should be identical
        assert_eq!(address1, address2);
        assert_eq!(address1.address_type(), AddressType::P2PKH);
    }

    #[test]
    fn test_different_address_types() {
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);

        let mainnet_addr = AddressGenerator::from_public_key_with_type(
            &keypair.public, 
            AddressType::P2PKH
        );
        let testnet_addr = AddressGenerator::from_public_key_with_type(
            &keypair.public, 
            AddressType::TestnetP2PKH
        );

        // Same hash160, different types
        assert_eq!(mainnet_addr.hash160(), testnet_addr.hash160());
        assert_ne!(mainnet_addr.address_type(), testnet_addr.address_type());
        assert_ne!(mainnet_addr.version_byte(), testnet_addr.version_byte());
    }

    #[test]
    fn test_address_verification() {
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let address = AddressGenerator::from_public_key(&keypair.public);

        // Verify correct address
        assert!(AddressGenerator::verify_address(&keypair.public, &address));

        // Create wrong address
        let wrong_address = Address::new([1u8; 20], AddressType::P2PKH);
        assert!(!AddressGenerator::verify_address(&keypair.public, &wrong_address));
    }

    #[test]
    fn test_known_test_vector() {
        // Known public key for deterministic testing
        let public_key_bytes = [
            0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d,
            0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73,
            0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6,
            0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29
        ];

        let address = AddressGenerator::from_public_key_bytes(&public_key_bytes);
        
        // Address should be deterministic for this input
        assert_ne!(address.hash160(), &[0u8; 20]);
        println!("Test vector address hash160: {:02x?}", address.hash160());
    }
}