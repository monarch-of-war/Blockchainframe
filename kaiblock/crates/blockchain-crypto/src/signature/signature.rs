use serde::{Deserialize, Serialize};
use std::fmt;
use crate::{CryptoError, Result};

///ed25519 signature wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature([u8; 64]);


impl Signature {
	///create signature from 64 bytes
	pub from_bytes(bytes: [u8; 64]) -> Self{
		Self(bytes)
	}


	///create signature from byte slice
	pub fn from_slice(slice: &[u8]) -> Result<Self> {
		if slice.len() != 64 {
			return Err(CryptoError::InvalidSignature);
		}

		let mut bytes = [0u8;64];
		bytes.copy_from_slice(slice);
		Ok(Self(bytes))
	}


	///get the raw bytes of the signature
	pub fn to_bytes(&self) -> [u8;64] {
		self.0
	}


	///convert to hex string
	pub fn to_hex(&self) -> String{
		hex::encode(self.0)
	}


	///get signature as byte slice
	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}


impl fmt::Display for Signature {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_hex())
	}
}


impl AsRef<[u8]> for Signature{
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}


impl From<[u8; 64]> for Signature{
	fn from(bytes: [u8;64]) -> Self {
		Self(bytes)
	}
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_creation() {
        let bytes = [42u8; 64];
        let sig = Signature::from_bytes(bytes);
        assert_eq!(sig.to_bytes(), bytes);
    }

    #[test]
    fn test_signature_hex_conversion() {
        let bytes = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let mut full_bytes = [0u8; 64];
        full_bytes[..8].copy_from_slice(&bytes);
        
        let sig = Signature::from_bytes(full_bytes);
        let hex = sig.to_hex();
        let restored = Signature::from_hex(&hex).unwrap();
        
        assert_eq!(sig, restored);
    }

    #[test]
    fn test_invalid_signature_length() {
        let result = Signature::from_slice(&[0u8; 32]);
        assert!(result.is_err());
        
        let result = Signature::from_slice(&[0u8; 128]);
        assert!(result.is_err());
    }

    #[test]
    fn test_signature_display() {
        let sig = Signature::from_bytes([0u8; 64]);
        let display = format!("{}", sig);
        assert_eq!(display, "0".repeat(128));
    }
}