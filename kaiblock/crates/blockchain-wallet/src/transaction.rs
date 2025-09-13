use crate::{
    WalletError,
    WalletKeyPair,
    Address,
}
use blockchain_core::transaction::Transaction;
use ed25519_dalek::Signature;
use bincode;


pub struct WalletTransaction;



impl WalletTransaction {
    pub fn new (sender: &WalletKeyPair, recipient: &str, amount: u64) -> Result<Transaction, WalletError> {
        let recipient_bytes = Address::validate(recipient)?;
        let mut tx = Transaction::new(sender.public_key_bytes(), recipient_bytes, amount);
        let signature: Signature = sender.sign(&tx.hash());
        tx.signature = Some(signature.to_bytes().to_vec());
        Ok(tx)
    }
}