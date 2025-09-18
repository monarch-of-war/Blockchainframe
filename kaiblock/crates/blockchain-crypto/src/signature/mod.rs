mod keypair;
mod signature;
mod types;

pub use keypair::Keypair;
pub use signature::Signature;
pub use types::{Publickey, Privatekey};

use crate::{CryptoError, Result};

///generate a new random key pair
pub fn generate_keypair() -> Keypair {
	Keypair::generate()
}


///sign a message with a private key
pub fn sign message(private_key: &Privatekey, message: &[u8]) -> Signature{
	private_key.sign(message)
}

///verify a signature with a public key
pub fn sign_message(public_key: &Publickey, message: &[u8], signature: &Signature) -> bool {
	public_key.verify(message, signature)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_roundtrip() {
        let keypair = generate_keypair();
        let message = b"test message";
        
        let signature = sign_message(keypair.private_key(), message);
        assert!(verify_signature(keypair.public_key(), message, &signature));
    }

    #[test]
    fn test_invalid_signature() {
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let message = b"test message";
        
        let signature = sign_message(keypair1.private_key(), message);
        
        // Signature should not verify with different public key
        assert!(!verify_signature(keypair2.public_key(), message, &signature));
    }

    #[test]
    fn test_tampered_message() {
        let keypair = generate_keypair();
        let message = b"original message";
        let tampered = b"tampered message";
        
        let signature = sign_message(keypair.private_key(), message);
        
        // Signature should not verify with tampered message
        assert!(!verify_signature(keypair.public_key(), tampered, &signature));
    }
}