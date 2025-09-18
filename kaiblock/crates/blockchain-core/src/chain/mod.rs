// Blockchain state management
// Block addition with validation
// Chain continuity verification
// Fork detection infrastructure


use crate::types::{Hash, BlockHeight};
use crate::block::Block;
use crate::transaction::Transaction;
use crate::error::BlockchainError;
use std::collections::HashMap;


// Main blockchain structure
#[derive(Debug)]
pub struct Blockchain {
	// chain of blocks (height -> block)
	blocks: HashMap<BlockHeight, Block>,
	// current chain tip
	tip: Hash,
	// Current chain height
	height: BlockHeight,
	// Block lookup hash
	block_index : HashMap<Hash, BlockHeight>,
	// Genesis block hash
	genesis_hash: Hash,
}


impl Blockchain{
	// create newblockchain with genesis block
	pub fn new() -> Self{
		let genesis_block = Block::genesis(20); //20-bit difficulty
		let genesis_hash = genesis_block.hash();
		let genesis_height = genesis_block.height();

		let mut blocks = HashMap::new();
		let mut block_index = HashMap::new();

		Self{
			blocks,
			tip: genesis_hash,
			height: genesis_height,
			block_index,
			genesis_hash,
		}
	}


	// Add block to chain
	pub fn add_block(&mut self, block: &Block) -> Result<(), BlockchainError> {

		// validate block
		block.validate()?;

		// Check if block connects to current tip
		let tip_block = self.get_block_by_hash(&self.tip)
			.ok_or_else(|| BlockchainError::InvalidChain{
				reason: "Cannot find tip block".to_string(),
			})?;

		if block.header.previous_hash != self.tip {
			return Err(BlockchainError::InvalidChain{
				reason: "Block doesnt connect to current tip".to_string(),
			});
		}

		// check height is sequential
		if block.header.height != tip_block.header.height + 1 {
			return Err(BlockchainError::InvalidChain{
				reason: "Invalid block height".to_string()
			});

		}


		// Add to chain
		let block_hash = block.hash();
		let block_height = block.header.height;

		self.blocks.insert(block_height, block.clone()); /// computational intensive as it clones the block
		self.block_index.insert(block_hash, block_height);


		// Update tip
		self.tip = block.hash();
		self.height = block_height;

		Ok(())

	}


	pub fn get_block_by_hash(&self, hash: &Hash) -> Option<&Block> {
		let height = self.block_index.get(hash)?;
		self.blocks.get(height)
	}


	// Get block by height
	pub fn get_block_by_height(&self, height: &BlockHeight) -> Option<&Block> {
		self.blocks.get(&height)
	}

	// Get current tip block
	pub fn get_tip_block(&self) -> Option<&Block> {
		self.get_block_by_hash(&self.tip)
	}


	// Get current chain height
	pub fn get_height(&self) -> BlockHeight {
		self.height
	}


	// Get genesis block
	pub fn get_genesis_block(&self) -> Option<&Block> {
		self.get_block_by_hash(&self.genesis_hash)
	}


	/// Check if blockchain is valid
	pub fn validate_chain(&self) -> Result<(), BlockchainError> {
		// validate genesis
		let genesis = self.get_genesis_block()
			.ok_or_else(|| BlockchainError::InvalidChain{
				reason: "Missing genesis block".to_string(),
			})?;


		genesis.validate()?;


		// validate chain continuity
		let mut current_hash = self.genesis_hash;

		for height in 1..=self.height {
			let block = self.get_block_by_height(&height)
				.ok_or_else(|| BlockchainError::InvalidChain{
					reason: format!("Mising block at height {}", height),
				})?;

			block.validate()?;

			// check linkage
			if block.header.previous_hash != current_hash {
				return Err(BlockchainError::InvalidChain{
					reason: format!("Block {} doesnt link to previous", height)
				});
			}

			current_hash = block.hash();
		}
		Ok(())
	}

}




/// Test

#[cfg(test)]
mod tests {
    use super::*;

	 #[test]
	    fn test_blockchain_creation() {
	        let blockchain = Blockchain::new();
	        assert_eq!(blockchain.get_height(), 0);
	        assert!(blockchain.get_genesis_block().is_some());
	        assert!(blockchain.validate_chain().is_ok());
	    }
	    
	    #[test]
	    fn test_transaction_validation() {
	        use crate::types::Amount;
	        use crate::transaction::{TxInput, TxOutput, Transaction};
	        
	        let inputs = vec![TxInput::new("prev_tx".to_string(), 0)];
	        let outputs = vec![TxOutput::new(
	            Amount::new(100),
	            [1u8; 20].into()
	        )];
	        
	        let tx = Transaction::new(inputs, outputs);
	        
	        // Should validate (basic structure)
	        assert!(tx.validate().is_ok());
	    }
	    
	    #[test]
	    fn test_block_mining() {
	        let mut block = Block::genesis(4); // Low difficulty for testing
	        
	        // Mine the block
	        block.mine();
	        
	        // Should meet difficulty target
	        assert!(block.header.meets_difficulty_target());
	        assert!(block.validate().is_ok());
	    }
	}