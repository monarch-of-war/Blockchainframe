use ed25519_dalek::{Keypair as DalekKeypair, PublicKey, SecretKey};
use rand::rngs::OsRng;

pub struct MyKeypair {
    pub public: PublicKey,
    pub private: SecretKey,
}

pub fn generate_keypair() -> DalekKeypair {
    let mut csprng = OsRng{};
    DalekKeypair::generate(&mut csprng)
}