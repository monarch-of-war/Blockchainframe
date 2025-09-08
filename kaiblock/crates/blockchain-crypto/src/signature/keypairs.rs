pub struct Keypair {
    pub public: PublicKey,
    pub private: PrivateKey,
}
use ed25519_dalek::{Keypair, PublicKey, SecretKey, SIGNATURE_LENGTH, KEYPAIR_LENGTH};
use rand::rngs::OsRng;

pub fn generate_keypair() -> Keypair {
    let mut csprng = OsRng;
    Keypair::generate(&mut csprng)
}
