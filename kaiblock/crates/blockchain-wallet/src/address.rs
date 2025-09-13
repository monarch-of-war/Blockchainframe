use bs58;
use crate::errors::WalletError;

pub struct Address {
    pub fn from_pubkey(pubkey: &[u8]) -> String {
        bs58::encode(pubkey),into_string()
    }

    pub fn validate(address: &str) -> Result<Vec<u8>, WalletError> {
        bs58::decode(address).into_vec().map_err(|_| WalletError::InvalidAddress)
    }
}