use crate::types::{Hash, BlockHeight, Timestamp, current_timestamp};
use crate::transaction::Transaction;
use crate::error::BlockchainError;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};




/// Block header containing metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub struct BlockHeader {
	//versioning for protocol upgrade
	pub version: u32,
	//hash of previous block
	pub previous_hash: Hash,
	//merkle root of transactions
	pub merkle_root: Hash,
	//block creation timestamp
	pub timestamp: Timestamp,
	//difficulty target
	pub difficulty: u32,
	//proof of work nonce
	pub nonce: u64,
	//block height in chain
	pub height: BlockHeight,
}


impl BlockHeader{
	pub fn new(
		previous_hash: Hash,
		merkle_root:Hash,
		difficulty: u32,
		height: BlockHeight,
		) -> Self{
		Self{
			version: 1,
			previous_hash,
			merkle_root,
			timestamp: current_timestamp(),
			difficulty,
			nonce: 0,
			height,
		}
	}


	// 1. hash the header
	// 2. Check if it meets the difficulty target
		// this is by calling a count leading zero function(the number of zeros should be the number of bits(hash consists of bytes thus 1 byte = 8 bits))
		// if not change the nounce value and iterate the process

	//calculate header hash for proof of work
	pub fn hash(&self) -> Hash{
		let serialized = bincode::serialize(self).unwrap();
		let mut hasher = Sha256::new();
		hasher.update(&serialized);
		hasher.finalize().into()
	}



	// check if difficulty tartge has been met
	pub fn meets_difficulty_target(&self) ->bool{
		let hash = self.hash();
		let leading_zeros = self.count_leading_zeros(&hash);
		leading_zeros >= self.difficulty
	}


	//count leading zero bits in hash
	fn count_leading_zeros(&self, hash: &Hash) -> u32 {
		let mut zeros = 0;

		for &byte in hash {
			if byte == 0 {
				zeros+= 8;

			}else {
				zeros += byte.leading_zeros();
				break
			}
		}

		zeros
	}
}


/// Complete block with header and transactions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
	//Block header
	pub header: BlockHeader,
	//block transactions
	pub transactions: Vec<Transaction>,
}


impl Block{

	//create new block

	pub fn new(
		previous_hash: Hash,
		transactions: Vec<Transaction>,
		difficulty: u32,
		height: BlockHeight,
		) -> Self{
		let merkle_root = Self.calculate_merkle_root(&transactions);
		let header = BlockHeader::new(previous_hash, merkle_root, difficulty, height);
		Self{
			header,
			transactions,
		}
	}

	// this is the tree like structure of transaction hashes which will be usefull for light nodes not tocarry the whole load of 
	// transactions thus only include the block headers but with thesame level of trust that all transactions in a block are valid
	// and thus trust is upheld in cases of a full node and a light node.



	// fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
	// 	if transactions.is_empty() {
	// 		return [0u8; 32];
	// 	}


	// 	//get transaction hashes

	// 	let mut hashes : Vec<Hash> = transactions.iter()
	// 		.map(|tx| tx.hash())
	// 		.collect();

	// 	//build merkle tree

	// 	while hashes.len() > 1{
	// 		let mut next_level = Vec::new();

	// 		//process pairs
	// 		for chunk in hashes.chunks(2){
	// 			let combined_hash = match chunk {
	// 				[left, right] => Self::combine_hashes(left, right),
	// 				[single] => Self::combine_hashes(single, single), //duplicate if odd

	// 				_ => unreachable!(),
	// 			};

	// 			next_level.push(combined_hash);
	// 		}

	// 		hashes = next_level;
	// 	}

	// 	hashes[0]
	// }


	// fn combine_hashes(left: &Hash, right: &Hash)->Hash {
	// 	let mut hasher = Sha256::new();
	// 	hasher.update(left);
	// 	hasher.update(right);
	// 	hasher.finalize().into()
	// }


	// --------------------------------------------------
	///////////////////////////////////////////////////////////
	//////////////////////////////////////////////////////////

	use blockchain_crypto::hash::{sha256, double_sha256, MerkleTree, merkle_tree_from_data, Hash256},

	fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
		let merkle_tree = merkle_tree_from_data(&transactions)

		merkle_tree.root();


	}



	//////////////////////////////////////////////////////
	/////////////////////////////////////////////////////
	////////////////////////////////////////////////////
	// --------------------------------------------------

	//get block hash
	pub fn hash(&self) -> Hash {
		self.header.hash()
	}

	//get block heighr
	pub fn height(&self) -> BlockHeight {
		self.header.height
	}


// validate block structure and content
	pub fn validate(&self) -> Result<(), BlockchainError> {
		//validate all transactions
		for tx in &self.transactions{
			tx.validate().map_err(|e| BlockchainError::InvalidBlock{
				reason: format!("Invalid transaction: {}", e)
			})?;
		}

		// check merkle root
		let calculated_merkle_root = Self::calculate_merkle_root(&self.transactions);
		if calculated_merkle_root != self.header.merkle_root{
			return Err(BlockchainError::InvalidBlock{
				reason: "Merkle root mismatch".to_string(),
			});
		}


		// check proof of work
		if !self.header.meets_difficulty_target(){
			return Err(BlockchainError::InvalidBlock{
				reason: "Block does not meet difficulty target".to_string(),
			});
		}


		// check for coinbase transaction(first transaction should be coinbase)
		if !self.transactions.is_empty() && !self.transactions[0].is_coinbase(){
			return Err(BlockchainError::InvalidBlock{
				reason: "First transactio must be coinbase".to_string(),
			});
		}



		// check for multiple coinbase transactions
		let coinbase_count = self.transactions.iter()
			.filter(|tx| tx.is_coinbase())
			.count();

		if coinbase_count != 1{
			return Err(BlockchainError::InvalidBlock{
				reason: format!("Block must have exactly one coinbase transaction, found {}", coinbase_count)
			});
		}

		Ok(())

	}


	// Mine blcok by finding valid nonce
	pub fn mine(&mut self){
		while !self.header.meets_difficulty_target(){
			self.header.nonce += 1;
		}
	}
}