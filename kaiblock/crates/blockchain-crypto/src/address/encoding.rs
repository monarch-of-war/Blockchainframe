use super::types::{Address, AddressType, AddressError};
use sha2::{Sha256, Digest};

/// Base58 encoder/decoder for Bitcoin-style addresses
pub struct Base58Encoder;

impl Base58Encoder {
    const BASE58_CHARS: &'static [u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    /// Encode address to Base58Check format
    pub fn encode(address: &Address) -> Result<String, AddressError> {
        let payload = address.to_bytes(); // version + hash160
        let checksum = Self::calculate_checksum(&payload);
        
        let mut full_payload = payload;
        full_payload.extend_from_slice(&checksum[0..4]); // First 4 bytes of checksum
        
        Ok(Self::base58_encode(&full_payload))
    }

    /// Decode Base58Check address string
    pub fn decode(encoded: &str) -> Result<Address, AddressError> {
        let decoded = Self::base58_decode(encoded)
            .map_err(|e| AddressError::EncodingError { details: e })?;

        if decoded.len() != 25 {
            return Err(AddressError::InvalidLength {
                expected: 25,
                actual: decoded.len(),
            });
        }

        // Split into payload and checksum
        let (payload, checksum) = decoded.split_at(21);
        
        // Verify checksum
        let calculated_checksum = Self::calculate_checksum(payload);
        if checksum != &calculated_checksum[0..4] {
            return Err(AddressError::InvalidChecksum);
        }

        // Parse address type and hash160
        let version_byte = payload[0];
        let address_type = Self::version_byte_to_address_type(version_byte)?;
        
        let mut hash160 = [0u8; 20];
        hash160.copy_from_slice(&payload[1..]);

        Ok(Address::new(hash160, address_type))
    }

    /// Calculate double SHA-256 checksum
    fn calculate_checksum(payload: &[u8]) -> [u8; 32] {
        let first_hash = Sha256::digest(payload);
        let second_hash = Sha256::digest(&first_hash);
        second_hash.into()
    }

    /// Convert version byte to AddressType
    fn version_byte_to_address_type(version_byte: u8) -> Result<AddressType, AddressError> {
        match version_byte {
            0x00 => Ok(AddressType::P2PKH),
            0x05 => Ok(AddressType::P2SH),
            0x6F => Ok(AddressType::TestnetP2PKH),
            0xC4 => Ok(AddressType::TestnetP2SH),
            _ => Err(AddressError::UnsupportedAddressType {
                type_byte: version_byte,
            }),
        }
    }

    /// Base58 encoding implementation
    fn base58_encode(data: &[u8]) -> String {
        if data.is_empty() {
            return String::new();
        }

        // Count leading zeros
        let leading_zeros = data.iter().take_while(|&&b| b == 0).count();

        // Convert to base58
        let mut num = num_bigint::BigUint::from_bytes_be(data);
        let base = num_bigint::BigUint::from(58u8);
        let mut result = Vec::new();

        while num > num_bigint::BigUint::from(0u8) {
            let remainder = &num % &base;
            result.push(Self::BASE58_CHARS[remainder.to_bytes_be()[0] as usize]);
            num /= &base;
        }

        // Add leading '1's for leading zeros
        let mut encoded = vec![b'1'; leading_zeros];
        encoded.extend(result.iter().rev());

        String::from_utf8(encoded).unwrap()
    }

    /// Base58 decoding implementation
    fn base58_decode(encoded: &str) -> Result<Vec<u8>, String> {
        if encoded.is_empty() {
            return Ok(Vec::new());
        }

        // Count leading '1's
        let leading_ones = encoded.chars().take_while(|&c| c == '1').count();

        // Decode from base58
        let mut num = num_bigint::BigUint::from(0u8);
        let base = num_bigint::BigUint::from(58u8);

        for c in encoded.chars().skip(leading_ones) {
            let digit = Self::BASE58_CHARS
                .iter()
                .position(|&b| b == c as u8)
                .ok_or_else(|| format!("Invalid Base58 character: {}", c))?;
            
            num = num * &base + num_bigint::BigUint::from(digit);
        }

        let mut decoded = num.to_bytes_be();
        
        // Add leading zeros for leading '1's
        let mut result = vec![0u8; leading_ones];
        result.append(&mut decoded);

        Ok(result)
    }
}

/// Simple hex encoder for development/debugging
pub struct HexEncoder;

impl HexEncoder {
    /// Encode address as hex string (version + hash160)
    pub fn encode(address: &Address) -> Result<String, AddressError> {
        let bytes = address.to_bytes();
        Ok(hex::encode(bytes))
    }

    /// Decode hex string to address
    pub fn decode(hex_string: &str) -> Result<Address, AddressError> {
        if hex_string.len() != 42 { // 21 bytes * 2 hex chars
            return Err(AddressError::InvalidLength {
                expected: 42,
                actual: hex_string.len(),
            });
        }

        let bytes = hex::decode(hex_string)
            .map_err(|_| AddressError::EncodingError {
                details: "Invalid hex encoding".to_string(),
            })?;

        let version_byte = bytes[0];
        let address_type = Base58Encoder::version_byte_to_address_type(version_byte)?;

        let mut hash160 = [0u8; 20];
        hash160.copy_from_slice(&bytes[1..]);

        Ok(Address::new(hash160, address_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::conversion::AddressGenerator;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_base58_round_trip() {
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let address = AddressGenerator::from_public_key(&keypair.public);

        // Encode to Base58
        let encoded = Base58Encoder::encode(&address).unwrap();
        println!("Base58 encoded: {}", encoded);

        // Decode back
        let decoded = Base58Encoder::decode(&encoded).unwrap();

        // Should match original
        assert_eq!(address, decoded);
    }

    #[test]
    fn test_hex_round_trip() {
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let address = AddressGenerator::from_public_key(&keypair.public);

        // Encode to hex
        let encoded = HexEncoder::encode(&address).unwrap();
        println!("Hex encoded: {}", encoded);

        // Decode back
        let decoded = HexEncoder::decode(&encoded).unwrap();

        // Should match original
        assert_eq!(address, decoded);
    }

    #[test]
    fn test_invalid_base58() {
        // Invalid character
        let result = Base58Encoder::decode("123456789O"); // 'O' not in Base58
        assert!(result.is_err());

        // Invalid checksum
        let result = Base58Encoder::decode("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        assert!(result.is_err());
    }
}