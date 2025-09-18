use super::{PublicKey, PrivateKey, Signature};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use crate::Result;


/// Ed25519 key pair for signing and verification
#[derive(Debug, Clone, PartialEq)]
pub struct Keypair{
	private_key: PrivateKey,
	public_key: PublicKey,
}

impl Keypair{
	///Generate a new rndom keypair
	pub fn generate() ->Self{
		let private_key = SigningKey::generate(&mut OsRng);
		let public_key = private_key.verify_key();

		Self{
			private_key: PrivateKey::from(private_key),
			public_key: PublicKey::from(public_key),
		}
	}

	///Create a key_pair from private_key bytes
	pub fn from_private_bytes(bytes: &[u8]) -> Result<Self>{
		let private_key = PrivateKey::from_bytes(bytes)?;
		Ok(Self::from_private_key(private_key))
	}

	///create a pair from a hex-encoded private key
	pub fn from_private_hex(hex_str: &str) -> Result<Self> {
		let private_key = PrivateKey::from_hex(hex_str)?;
		Ok(Self::from_private_key(private_key))
	}

	///get private key
	pub fn private_key(%self) -> &PrivateKey {
		&self.private_key
	}


	///get public key
	pub fn public_key(&self) -> PublicKey{
		&self.public_key
	}


	///Sign a message with this key_pair

	pub sign(&self, message: &[u8]) -> Signature {
		self.private_key.sign(message)
	}

	///verify a signature with this key pair's public key
	pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
		self.public_key.verify(message, signature)
	}


	///get private key as bytes
	pub fn private_key_bytes(&self) -> [u8;32] {
		self.private_key.to_bytes()
	}

	///get the public key as bytes
	pub fn public_key_bytes(&self) -> [u8; 32] {
		self.public_key.to_bytes()
	}


	///export private key as hex string
	pub fn export_private_key(&self) -> String{
		self.private_key.to_hex()
	}


	///export public key as hex string
	pub fn export_public_key(&self) -> String{
		self.public_key.to_hex()
	}


}



/// Serializable key pair data (for secure storage)
#[derive(Serialize, Deserialize)]
pub struct SerializableKeyPair{
	pub private_key_hex: String,
	pub public_key_hex: String,
}


impl From<&Keypair> for SerializableKeyPair{
	fn from(keypair: &Keypair) -> Self {
		Self{
			private_key_hex: keypair.export_private_key(),
			public_key_hex: keypair.export_public_key(),
		}
	}
}


impl TryFrom<SerializableKeyPair> for Keypair {
	type Error = crate::CryptoError;

	fn try_from(data: SerializableKeyPair) -> Result<Self> {
		Keypair:;from_private_hex(&data.private_key_hex)
	}
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        
        // Test that we can sign and verify
        let message = b"test message";
        let signature = keypair.sign(message);
        assert!(keypair.verify(message, &signature));
    }

    #[test]
    fn test_keypair_from_private_key() {
        let keypair1 = KeyPair::generate();
        let private_bytes = keypair1.private_key_bytes();
        
        let keypair2 = KeyPair::from_private_bytes(&private_bytes).unwrap();
        
        assert_eq!(keypair1, keypair2);
    }

    #[test]
    fn test_keypair_serialization() {
        let keypair = KeyPair::generate();
        let serializable: SerializableKeyPair = (&keypair).into();
        let restored = KeyPair::try_from(serializable).unwrap();
        
        assert_eq!(keypair, restored);
    }

    #[test]
    fn test_keypair_hex_roundtrip() {
        let keypair = KeyPair::generate();
        let private_hex = keypair.export_private_key();
        let restored = KeyPair::from_private_hex(&private_hex).unwrap();
        
        assert_eq!(keypair, restored);
    }

    #[test]
    fn test_sign_verify_different_messages() {
        let keypair = KeyPair::generate();
        let message1 = b"message 1";
        let message2 = b"message 2";
        
        let signature1 = keypair.sign(message1);
        let signature2 = keypair.sign(message2);
        
        // Signatures should be different
        assert_ne!(signature1, signature2);
        
        // Each should verify with correct message
        assert!(keypair.verify(message1, &signature1));
        assert!(keypair.verify(message2, &signature2));
        
        // But not with wrong message
        assert!(!keypair.verify(message1, &signature2));
        assert!(!keypair.verify(message2, &signature1));
    }

    #[test]
    fn test_keypair_uniqueness() {
        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::generate();
        
        // Should generate different keys
        assert_ne!(keypair1.private_key_bytes(), keypair2.private_key_bytes());
        assert_ne!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
    }
}