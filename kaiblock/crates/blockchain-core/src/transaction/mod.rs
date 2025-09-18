use crate::types::{Hash, Amount, Timestamp, Address, current_timestamp};
use crate::error::BlockchainError;
use serde::{Serialize, Deserialize};
use uuid::Uuid;


// Transaction Module (transaction/)

// Transaction, TxInput, TxOutput structures
// Coinbase transaction creation for mining rewards
// Fee calculation and validation logic
// Transaction serialization and hashing

//transaction identifier
pub type TxId = String;

//transaction input - references previous output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxInput {
	//previous transaction ID
	pub prev_tx_id: TxId,

	//output index of prev tx
	pub prev_output_index: u32,

	//signature proving ownership
	pub signature: Vec<u8>,

	//public key for verification
	pub public_key:  Vec<u8>,
}


impl TxInput{
	pub fn new(prev_tx_id: TxId, prev_output_index: u32) ->Self{
		Self{
			prev_tx_id,
			prev_output_index,
			signature: Vec::new(),
			public_key: Vec::new(),
		}
	}


	pub fn sign(&mut self, signature: Vec<u8>, public_key: Vec<u8>) {
		self.signature = signature;
		self.public_key = public_key;
	}
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutput {

	//Amount to transfer
	pub amount: Amount,

	//recipient address
	pub recipient: Address,
}


impl TxOutput {
	pub fn new(amount: Amount, recipient: Address) ->Self{
		Self {amount, recipient}
	}
}



//complete transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
	//Unique transaction identifier
	pub id: TxId,
	//transaction imputs
	pub inputs: Vec<TxInput>,
	//transaction outputs
	pub outputs: Vec<TxOutput>,
	//transaction timestamp
	pub timestamp: Timestamp,
	//transaction version for future upgrades
	pub version: u32,
}

impl Transaction {
	//Create new transaction
	pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
		let id = Uuid::new_v4().to_string();
		Self{
			id,
			inputs,
			outputs,
			timestamp: current_timestamp(),
			version: 1,
		}
	}

	pub fn coinbase(recipient: Address, reward: Amount, block_height: u64)-> Self{
		let outputs = vec![TxOutput::new(reward, recipient)];
		let mut tx = Self{
			id: Uuid::new_v4().to_string(),
			inputs: Vec::new(), //coinbase has no inputs
			outputs,
			timestamp: current_timestamp(),
			version: 1,
		};

		tx.id = format!("Coinbase-{}", block_height);
		tx
	}


	//culculate total input aount
	pub fn total_input_amount(&self) -> Amount {
		//in real implementation, we would look up UTXO set
		//for now, return zero for coinbase, placeholder for others
		if self.inputs.is_empty(){
			Amount::ZERO

		}else {
			Amount::new(1000) //placeholder
		}
	}


	//culculate total output amount
	pub fn total_output_amount(&self) ->Amount{
		self.outputs.iter()
		.map(|output| output.amount)
		.fold(Amount::ZERO, |acc, amount| acc + amount)
	}


	pub fn fee(&self) -> Amount{
		let input_total = self.total_input_amount();
		let output_total = self.total_output_amount();

		if input_total.value() >= output_total.value() {
			Amount::new(input_total.value() - output_total.value())

		}else {
			Amount::ZERO
		}

	}


	//validate transaction structure
	pub fn validate(&self) -> Result<(), BlockchainError> {
		//check for empty inputs (except coinbase)
		if self.inputs.is_empty() && !self.is_coinbase() {
			return Err(BlockchainError::InvalidTransaction{
				reason: "Non-coinbase transaction must have inputs".to_string(),
			});
		}

		//check for empty outputs
		if self.outputs.is_empty(){
			return Err(BlockchainError::InvalidTransaction{
				reason: "Transaction must have outputs".to_string(),
			});
		}

		//check output aount are positive
		for output in &self.outputs{
			if output.amount.is_zero(){
				return Err(BlockchainError::InvalidTransaction{
					reason: "Output amount must be positive".to_string(),
				});
			}
		}

		//Check for input/output balance(except coinbase)

		if !self.is_coinbase() {
			let input_total = self.total_input_amount();
			let output_total = self.total_output_amount();

			if input_total.value() < output_total.value() {
				return Err(BlockchainError::InsufficientFunds{
					available: input_total.value(),
					required: output_total.value(),
				});
			}
		}

		Ok(())
	}


	//check if this is a coinbase transaction
	pub fn is_coinbase(&self) -> bool {
		self.inputs.is_empty()
	}


	// Serialize transaction for hashing
	pub fn serialize(&self) ->Vec<u8> {
		bincode::serialize(self).unwrap_or_default()
	}


	// calculate the transacion hash

	pub fn hash(&self) -> Hash {
		use sha2::{Sha256, Digest};
		let data = self.serialize();
		let mut hasher = Sha256::new();
		hasher.update(&data);
		hasher.finalize().into()
	}



}


