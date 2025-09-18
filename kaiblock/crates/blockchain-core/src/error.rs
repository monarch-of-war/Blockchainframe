use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
	#[error("Transaction validation failed: {reason}")]
	InvalidTransaction {reason: String},

	#[error("Block validationfailed: {reason}")]
	InvalidBlock {reason: String},


	#[error("Insufficient funds: have{available}, need {required}")]
	InsufficientFunds {available: u64, required: u64},


	#[error("Transaction not found: {tx_id}")]
	TransactionNotFound {tx_id: String},


	#[error("Block not found: {block_hash}")]
	BlockNotFound {block_hash: String},


	#[error("Invalid chain: {reason}")]
	InvalidChain {reason: String},

	#[error("Serialization error: {details}")]
	SerializationError {details: String},

	#[error("Cryptogtaphic error: {details}")]
	CryptoError {details: String},

	#[error("Fork handling error: {details}")]
	ForkError {details: String},
}


