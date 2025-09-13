// blockchain-core/src/ledger/transaction.rs

use serde::{Serialize, Deserialize};
use blockchain_crypto::hash::sha256;
use blockchain_crypto::signature::{PublicKey, Signature, sign_message, verify_signature};
use blockchain_crypto::address::Address;




// Uses serde + bincode for serialization.

// Each input references a past output (prev_txid + index).

// sign() signs the transaction ID and attaches both the signature + public key.

// verify() ensures all signatures are valid.

// Transaction IDs are deterministic hashes of the serialized struct.
/// A single input to a transaction (references a previous output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub prev_txid: Vec<u8>,   // hash of previous transaction
    pub output_index: u32,    // index of the output being spent
    pub signature: Option<Signature>, // filled after signing
    pub public_key: Option<PublicKey>, // prove ownership
}

/// A single output of a transaction (where funds go).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: u64,       // amount of coins
    pub address: Address, // recipient’s address
}

/// Core transaction structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub timestamp: u64,
}

impl Transaction {
    /// Serialize and hash the transaction (for ID).
    pub fn txid(&self) -> Vec<u8> {
        let encoded = bincode::serialize(self).unwrap();
        sha256(&encoded)
    }

    /// Sign all inputs with the given private key.
    pub fn sign(&mut self, private_key: &ed25519_dalek::Keypair) {
        let message = self.txid();
        for input in &mut self.inputs {
            let sig = sign_message(&message, private_key);
            input.signature = Some(sig);
            input.public_key = Some(private_key.public.clone());
        }
    }

    /// Verify signatures for all inputs.
    pub fn verify(&self) -> bool {
        let message = self.txid();
        self.inputs.iter().all(|input| {
            if let (Some(sig), Some(pubkey)) = (&input.signature, &input.public_key) {
                verify_signature(&message, sig, pubkey)
            } else {
                false
            }
        })
    }
}
