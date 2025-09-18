pub mod address; 
pub mod hash;
pub mod signature;

use thiserror::Error;



///core xryptographic errors encountered
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CryptoError {
	#[error("Invalid key format: {0}")]
	InvalidKey(String),
	#[error("Siggnature verification failed")]
	InvalidSignature,
	#[error("invalid hash format: {0}")]
	InvalidHash(String),
	#[error("Address format error: {0}")]
	AddressError(String,
	#[error("serialization error: {0}")]
	SerializationError(String),
	#[error("Invalid merkle proof")]
	InvalidMerkleProof,
}


pub type Result<T> = std::result::Result<T, CryptoError>;

//re-export commonly used types
pub use address::{Address, AddressType};
pub use hash::{Hash256, MerkleTree, MerkleProof};
pub use signature::{Keypair, Publickey, Privatekey}