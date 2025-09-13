use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use crate::errors::WalletError;


pub struct WalletKeyPair{
    pub keypair: Keypair,
}


impl WalletKeyPair{
    pub fn genetate() -> Self{
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        Self{keypair}

    }

    pub fn from_secret(secret: &[u8]) -> Result<Self, WalletError> {
        let secret = SecretKey::from_bytes(secret).map_err(|_| WalletError::InvalidKey)?;
        let public = PublicKey::from(&secret);
        let keypair = Keypair{secret, public};
        Ok(Self {keypair})
    }

    pub fn sign(&self, message: &[u8]) -> Signature{
        self.keypair.sign(message)
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        self.keypair.public.verify(message, signature).is_ok()
    }

    // encoding of public key and private key
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.keypair.public.to_bytes(),to_vec()
    }

    pub fn secret_key_bytes(&self) ->Vec<u8> {
        self.keypair.secret.to_bytes().to_vec()
    }
}