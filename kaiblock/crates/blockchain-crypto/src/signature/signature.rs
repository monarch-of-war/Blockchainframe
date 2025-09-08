use ed25519_dalek::{Keypair, Signature, Signer};

pub fn sign_message(keypair: &Keypair, message: &[u8]) -> Signature {
    keypair.sign(message)
}
