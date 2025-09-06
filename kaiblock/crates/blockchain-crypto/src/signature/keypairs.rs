pub struct Keypair {
    pub public: PublicKey,
    pub private: PrivateKey,
}

pub struct PublicKey(Vec<u8>); // Placeholder for actual public key representation
pub struct PrivateKey(Vec<u8>); // Placeholder for actual private key representation

impl Keypair {
    pub fn new(public: Vec<u8>, private: Vec<u8>) -> Self {
        Keypair {
            public: PublicKey(public),
            private: PrivateKey(private),
        }
    }

    pub fn from_bytes(private: &[u8], public: &[u8]) -> Result<Self,KeypairError> {
        if private.len() != 32 || public.len() != 32 {
            return Err(KeypairError::InvalidKeyLength);
        }
        Ok(Keypair {
            public: PublicKey(public.to_vec()),
            private: PrivateKey(private.to_vec()),
        })

        
    }
    

}