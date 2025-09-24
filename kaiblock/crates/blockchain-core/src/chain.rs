use crate::types::*;
use crate::block::Block;
use crate::transaction::Transaction;
use crate::state::WorldState;
use crate::mempool::Mempool;
use crate::validation::{Validator, ValidationRules, BlockValidationContext};
use crate::{BlockchainError, Result};
use blockchain_crypto::{Address, Hash256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};


/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
	//network type
	pub network: NetworkType,
	//chain id
	pub chain_id: ChainId,
	//account model to use
	pub account_model: AccountModel,
	//genesis configuration
	pub genesis: GenesisConfig,
	//validation rules
	pub validation_rules: ValidationRules,
	//mining config
	pub mining: MiningConfig,
}

/// Genesis block configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig{
	//genesis coinbase recipient
	pub coinbase_recipient: Address,
	//reward
	pub genesis_reward: Amount,
	//initial accounts and balances(for account model)
	pub initial_accounts: HashMap<Address, Amount>,
	//genesis timestamp
	pub timestamp: Option:<i64>,
	//genesis_difficulty
	pub genesis_difficulty: Difficulty,
}


/// Gmining config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig{
	//mining reward amount
	pub block_reward: Amount,
	//target block tome in s
	pub target_block_time: u64,
	//max iterations b4 timeout
	pub max_mining_iterations: u64,
	//enable ming
	pub enable_mining: bool,
}

impl Default for ChainConfig {
	fn default() -> Self{
		network: NetworkType::Devnet,
		chain_id: 1,
		account_model: AccountModel::Hybrid,
		genesis: GenesisConfig{
			coinbase_recipient: Address::from_string("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa").unwrap_or_else(|_|{
				//fall back to a dummy if parsing failed
				let keypair = blockchain_crypto::signature::generate_keypair();
				blockchain_crypto::address::public_key_to_address(keypair.public_key, blockchain_crypto::AddressType::Base58)
			}),
			genesis_reward: 50_000_000, // 1	kai = 1_000_000 koins
			initial_accounts: HashMap::new(),
			timestamp: None,
			difficulty: 1,
		},

		validation_rules: ValidationRules::default(),
		mining: MiningConfig{
			block_reward: 25_000_000 //25 kais
			target_block_time: 600 //10 minutes
			max_mining_iterations: 1_000_000,
			enable_mining: true,
		},
	}
}



#[derive(Debug)]
pub struct Blockchain {
	///chain config
	config: ChainConfig,
	///current world state
	world_state: WorldState,
	///Block storage (hash-> block)
	blocks: HashMap<BlockId, Block>,
	///main chain(height -> block_id)
	main_chain: HashMap<BlockHeight, BlockId>,
	///current chain head
	chain_head: Option<BlockId>,
	///chain height
	height: BlockHeight,
	///transaction mempool
	mempool: Mempool,
	///validator
	Validator: Validator,
	///orphaned blocks(block_id -> block)
	orphaned_blocks: HashMap<BlockId, Block>,
}



impl Blockchain{
	///create new blockchain with configuration
	pub fn new(config: ChainConfig) -> Result<Self> {
		let world_state = WorldState::new(config.account_model);
		let validator = Validator::new(config.validation_rules.clone());
		let mempool = Mempool::default();

		let mut blockchain = Self {
			config,
			world_state,
			blocks: HashMap::new(),
			main_chain: HashMap::new(),
			chain_head: None,
			height: 0,
			mempool,
			validator,
			orphaned_blocks: HashMap::new(),
		};

		blockchain.create_genesis_block()?;

		Ok(blockchain)

	}


	///create genesis block
	fn create_genesis_block(&mut self) -> Result<()> {
		info!("creating genesis block");

		let genesis_config = &self.config.genesis;

		//create coinbase transaction
		let coinbase_tx = Transaction::new_coinbase(
			genesis_config.coinbase_recipient,
			genesis_config.genesis_reward,
			0, //genesis height
			);


		///create genesis block
		let mut genesis_block = Block::new(
			BlockId::genesis(),
			vec![coinbase_tx],
			genesis_config.difficulty,
			0, //genesis height
			self.config.chain_id,
			)?;

		//set custom timestamp if provided
		if let Some(timestamp) = genesis_config.timestamp {
			genesis_block.header.timestamp = Timestamp::from_unix_timestamp(timestamp);
		}


		//mine genesis block if needed
		if self.config.mining.enable_mining {
			info!("Mining genesis block...");
			let mined = genesis_block.mine(Some(self.config.mining.max_mining_iterations))?;

			if !mines {
				return Err(BlockchainError::InvalidBlock(
					"Failed o mine genesis block".to_string()
					));
			}


			info!("Genesis block mined with nonce {}", genesis_block.header.nonce);
		}

		let genesi_id = genesis_block.id();

		//add to chain
		self.blocks.insert(genesi_id, genesis_block.clone());
		self.main_chain.insert(0, genesi_id);
		self.chain_head = Some(genesis_id);
		self.height = 0;


		//apply genesis block to world state
		for tx in genesis_block.transactions() {
			self.world_state.apply_transaction(tx)?;
		}

		self.world_state.set_block_height(0);


		//initialize pre-funded accounts (for account model)

		for(address, balance) in &genesis_config.initial_accounts{
			let mut account_state = self.world_state.get_account(address).clone();
			account_state.add_balance(*balance)?;
			self.world_state.set_account(*address, account_state);

		}

		info!("Genesis block created: {}", genesi_id);
		Ok(())

	}


	pub fn add_block(&mut self, mut block: Block) -> Result<BlockId> {
		let block_id = block.id();
		let block_height = block.height();

		info!("Adding block {} at height {}", block_id, block_height);


		//check if block already exists
		if self.blocks.contains_key(&block_id){
			return Err(BlockchainError::InvalidBlock(
				"rblock already exists".to_string()
				));
		}


		//get previous block for validation
		let prev_block = if block.is_genesis() {
			None
		} else {
			self.blocks.get(&block.prev_hash())
		};


		//validate block
		let validation_ctx = BlockValidationContext{
			block: &block,
			prev_block,
			world_state: &self.world_state,
			rules: self.validator.rules(),
		};

		self.validator.validate_block(validation_ctx)



		//check if this block extends the main chain
		let extend_main_chain = match self.chain_head {
			Some(head_id) => block.prev_hash() == head_id,
			None => block.is_genesis(),
		};

		if extend_main_chain {
			//add to main chain
			self.add_to_main_chain(block)?;

		}else {
			//handle potential fork or orphan
			self.handle_fork(block)?;
		}

		Ok(block_id)
	}

	fn add_to_main_chain(&mut self, block: Block) -> Result<()> {
		let block_id = block.id();
		let block_height =  block.height();

		//apply block transaction to world state
		let mut new_state = self.world_state.clone();
		for tx in block.transactions() {
			new_state.apply_transaction(tx)?;
		}

		new_state.set_block_height(block_height);

		//remove transaction from mempool
		let tx_ids: Vec<TxId> = block.transactions().iter().map(|tx| tx.id());
		self.mempool.remove_transactions(&tx_ids);

		//update chain state
		self.blocks.insert(block_id, block);
		self.main_chain.insert(block_height, block_id);
		self.chain_head = Some(block_id);
		self.height = block_height;
		self.world_state = new_state;

		info!("Block {} added to main chain at height {}", block_id, block_height);
		Ok(())
	}


	///handle potential blockchain fork
	fn handle_fork(&mut self, block: Block) -> Result<()> {
		let block_id = block.id();
		let block_height = block.height();

		warn!("Potential fork detected with block {} at height {}", block_id, block_height);


		//check if previous block exists (might be orphan)
		if !self.blocks.contains_key(&block.prev_hash){
			info!("Adding orphan block: {}", block_id);
			self.orphaned_blocks.insert(block_id, block);
			return Ok(());
		}


		//TODO: IMPLEMENT PROPER FORK RESOLUTION
		//for now i store as orphan
		self.orphaned_blocks.insert(block_id, block);

		//try to process orphan blocks that might now be valid
		self.process_orphan_blocks()?;
	}

	//process orphan blocks that may be valid
	fn process_orphan_blocks(&mut self) -> Result<()> {
		let mut processed = Vec::new();

		//look for o_b whose parents are now available
		for (orphan_id, orphan_block) in &self.orphaned_blocks {
			if self.blocks.contains_key(&orphan_id.prev_hash()) {
				//this orphan can be processed
				processed.push(*orphan_id);
			}
		}


		//process the orphan blocks
		for orphan_id in processed {
			if let Some(orphaned_block) = self.orphaned_blocks.remove(&orphan_id) {
				info!("Processing orphan block: {}", orphan_id);
				self.handle_fork(orphan_block)?;
			}
		}

		Ok(())
	}


	//add transaction to mempool
	pub fn add_transaction(&mut self, transaction: Transaction) -> Result<TxId> {
		let tx_id = transaction.id();

		//check i transaction already in block
		if self.transaction_exists(&tx_id) {
			return Err(BlockchainError::InvalidTransaction(
				"Transaction already in blockchain".to_string()
				));
		}

		//add mempool
		self.mempool.add_transaction(transaction, &self.world_state)?;

		info!("Transaction {} added to mempool". tx_id);

		Ok(tx_id)
	}


	//check i transaction exists in blockchain
	pub fn transaction_exists(&self, tx_id: &TxId) -> bool {
		for block in self.blocks.values() {
			if block.get_transaction(tx_id).is_some() {
				return true;
			}
		}

		false
	}

	///get transaction by id (from blocks or mempool)
	pub fn get_transaction(&self, tx_id: &TxId) -> Option<&Transaction> {
		//first check blocks
		for block in self.blocks.values() {
			if let Some(tx) = block.get_transaction(tx_id) {
				return Some(tx)
			}
		}


		//then check mempool
		self.mempool.get_transaction(tx_id)
	}


	///get block by id
	pub fn get_block(&self, block_id: &BlockId) -> Option<&Block> {
		self.blocks.get(block_id)
	}


	///get block by height
	pub fn get_block_by_height(&self, height: &BlockHeight) -> Option<&Block> {
		self.main_chain.het(&height).and_then(|block_id| self.blocks.get(block_id))
	}


	///get current height
	pub fn height(&self) -> BlockHeight {
		self.height
	}


	//get world-state
	pub fn world_state(&self) -> &WorldState {
		&self.world_state
	}

	///get mempool
	pub fn mempool(&self) -> &Mempool {
		&self.mempool
	}

	///get mutable mempool
	pub fn mempool_mut(&mut self) -> &mut Mempool{
		&mut self.mempool
	}


	//mine a block
	pub fn mine_block(&mut self, miner_address: Address) -> Result<Block> {
		if !self.config.mining.enable_mining {
			return Err(BlockchainError::InvalidBlock(
				"Mining is disabled".to_string()
				));
		}

		info!("Mining a mew block for address: {}", miner_address);

		//get transactions from mempool
		let max_transactions = self.validator.rules().max_transactions_per_block;
		let max_size = self.validator.rules().max_block_size;
		let pending_txs = self.mempool.get_transaction_for_block(
			max_transactions,
			max_size,
			&self.world_state,
			);

		//create coinbase transaction
		let next_height = self.height + 1;
		let coinbase_tx = Transaction::new_coinbase(
			miner_address,
			self.config.mining,block_reward,
			next_height,
			);

		//combine coinbase with pending transactions
		let mut block_transactions = vec![coinbase_tx];
		block_transactions.extend(pending_txs);


		//get previous block hash
		let prev_hash = self.chain_head.unwrap_or_else(BlockId::genesis);

		//calculate next dificulty(simplified)
		let difficulty = self.calculate_next_difficulty()?;

		//create new block
		let mut new_block = Block::new(
			prev_hash,
			block_transactions,
			difficulty,
			next_height,
			self.config.chain_id,
			)?;

		//mine the block
		info!("Starting mining process...");
		let mining_start = std::time::Instant::now();
		let mined = new_block.mine(Some(self.config.mining.max_mining_iterations))?;

		if !mined{
			return Err(BlockchainError::InvalidBlock(
				"Failed to mine block within itration limit".to_string()
				));
		}

		let mining_time = mining_start.elapsed();

		info!("Block mined in {:?} with nonce: {}", mining_time, new_block.header.nonce);

		//add the mined block to the chain
		self.add_block(new_block.clone())?;

		Ok(new_block)

	}


	///calculate next block difficulty (simplified version)
	fn calculate_next_difficulty(&self) -> Result<Difficulty> {
		//for now, return a constant diff...in real implementation
		//adjustment will be based on block times.
		Ok(self.config.genesis.difficulty)
	}


	//get account balance
	pub fn get_balance(&self, address: &Address) -> Amount {
		self.world_state.get_balance(address)
	}

	//get account nonce
	pub get_nonce(&self, address: &Address) -> Nonce {
		self.world_state.get_nonce(address)
	}


	//get block statistics
	pub fn get_stats(&self) -> BlockchainStats {
		let total_transactions: usize = self.blocks.values()
			.map(|block| block.transaction_count())
			.sum();

		let total_supply = self.world_state.total_supply();
		let mempool_stats = self.mempool.get_stats();


		BlockchainStats {
			height: self.height,
			total_blocks: self.blocks.len(),
			total_transactions,
			total_supply,
			mempool_size: mempool_stats.transaction_count,
			orphaned_blocks: self.orphaned_blocks.len(),
			chain_head: self.chain_head,
		}
	}


	///validate entire chain consistencey
	pub fn validate_chain(&self) -> Result<()> {
		info!("Validating entire blockchain...");


		//collect blocks in height order
		let mut blocks_by_height: Vec<_> = self.main_chain.iter().collect();
		blocks_by_height.sort_by_key(|(height, _)| *height);

		let blocks: Vec<&Block>  = blocks_by_height.iter()
			.filter_map(|(_, block_id)| self.blocks.get(block_id))
			.collect();


		//create initial state for validation
		let initial_state = WorldState::new(self.config.account_model);

		//validation chain consistency
		crate::validation::validate_chain_consistency(&self.validator, &blocks, &initial_state)?;

		info!("Blockchain vvalidation completed successfully");
		Ok(())
	}


	//get block in height range
	pub fn get_block_range(&self, start_height: BlockHeight, end_height: BlockHeight) -> Vec<&Block> {
		(start_height..=end_height)
			.filter_map(|height| self.get_block_by_height(height))
			.collect()
	}

	//get recent blocks
	pub fn get_recent_blocks(&self, count: usize) ->Vec<&Block> {
		let start_height = self.height.saturating_sub(count as BlockHeight);
		self.get_block_range(start_height, self.height)
	}

	///update chain configuration
	pub update_config(&mut self, config: ChainConfig) -> Result<()> {
		info!("Updating blockchain configuration");

		//update validator rules
		self.validator.update_rules(config.validation_rules.clone());

		//update config
		self.config = config;

		Ok(())


	}


	//export chain data for backup/analysis
	pub fn export_chain_data(&self) -> ChainExport {
		let blocks: Vec<_> = (0..=self.height)
			.filter_map(|height| self.get_block_by_height(height))
			.cloned()
			.collect();

		ChainExport {
			config: self.config.clone(),
			blocks,
			world_state: self.world_state.clone(),
			height: self.height,
		}
	}

	//get fork information
	pub fn get_fork_info(&self) -> ForkInfo {
		ForkInfo{
			main_chain_height: self.height,
			orphan_blocks: self.orphaned_blocks.keys.clone().collect(),
			total_orphans: self.orphaned_blocks.len(),
		}
	}

}


///blockchain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
	pub height: BlockHeight,
	pub total_blocks: usize,
	pub total_transactions: usize,
	pub total_supply: Amount,
	pub mempool_size: usize,
	pub orphan_blocks: usize,
	pub chain_head: Option<BlockId>,
}


//chain export data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExport {
	pub config: ChainConfig,
	pub blocks: Vec<Block>,
	pub world_state: WorldState,
	pub height: BlockHeight,
}


///Fork information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkInfo {
	pub main_chain_height: BlockHeight,
	pub orphan_blocks: Vec<BlockId>,
	pub total_orphans: usize,
}

impl Default for Blockchain{
	fn default() -> Self {
		Self::new(ChainConfig::default()).expect("Failed to create default blockchain")
	}
}



////////////////////////TESTS\\\\\\\\\\\\\\\\\\\\\\\\
#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_crypto::{signature::generate_keypair, address::public_key_to_address, AddressType};

    #[test]
    fn test_blockchain_creation() {
        let config = ChainConfig::default();
        let blockchain = Blockchain::new(config).unwrap();
        
        assert_eq!(blockchain.height(), 0);
        assert!(blockchain.get_chain_head().is_some());
        assert_eq!(blockchain.blocks.len(), 1); // Genesis block
    }

    #[test]
    fn test_add_transaction() {
        let mut blockchain = Blockchain::default();
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        // Set up initial balance
        let mut account_state = blockchain.world_state.get_account(&addr1).clone();
        account_state.balance = 10000;
        blockchain.world_state.set_account(addr1, account_state);
        
        // Create and add transaction
        let tx = Transaction::new_account(addr1, addr2, 1000, 0, 21000, 20, vec![]);
        let tx_id = blockchain.add_transaction(tx).unwrap();
        
        assert!(blockchain.mempool.contains_transaction(&tx_id));
        assert_eq!(blockchain.mempool.len(), 1);
    }

    #[test]
    fn test_mine_block() {
        let mut blockchain = Blockchain::default();
        let miner_keypair = generate_keypair();
        let miner_address = public_key_to_address(miner_keypair.public_key(), AddressType::Base58);
        
        // Add a transaction to mempool first
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        // Set up balance for transaction
        let mut account_state = blockchain.world_state.get_account(&addr1).clone();
        account_state.balance = 10000;
        blockchain.world_state.set_account(addr1, account_state);
        
        let tx = Transaction::new_account(addr1, addr2, 1000, 0, 21000, 20, vec![]);
        blockchain.add_transaction(tx).unwrap();
        
        // Mine a block
        let initial_height = blockchain.height();
        let block = blockchain.mine_block(miner_address).unwrap();
        
        assert_eq!(blockchain.height(), initial_height + 1);
        assert_eq!(block.height(), initial_height + 1);
        assert!(block.transaction_count() >= 1); // At least coinbase
        assert_eq!(blockchain.mempool.len(), 0); // Transaction should be removed from mempool
    }

    #[test]
    fn test_get_balance() {
        let blockchain = Blockchain::default();
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        // Initially should have 0 balance
        assert_eq!(blockchain.get_balance(&address), 0);
        
        // Genesis coinbase recipient should have genesis reward
        let genesis_recipient = blockchain.config.genesis.coinbase_recipient;
        let expected_balance = blockchain.config.genesis.genesis_reward;
        assert_eq!(blockchain.get_balance(&genesis_recipient), expected_balance);
    }

    #[test]
    fn test_blockchain_stats() {
        let blockchain = Blockchain::default();
        let stats = blockchain.get_stats();
        
        assert_eq!(stats.height, 0);
        assert_eq!(stats.total_blocks, 1); // Genesis block
        assert_eq!(stats.total_transactions, 1); // Genesis coinbase
        assert!(stats.total_supply > 0);
        assert_eq!(stats.mempool_size, 0);
        assert_eq!(stats.orphan_blocks, 0);
        assert!(stats.chain_head.is_some());
    }

    #[test]
    fn test_chain_validation() {
        let blockchain = Blockchain::default();
        
        // Should validate successfully
        assert!(blockchain.validate_chain().is_ok());
    }

    #[test]
    fn test_get_transaction() {
        let blockchain = Blockchain::default();
        
        // Get genesis coinbase transaction
        let genesis_block = blockchain.get_chain_head().unwrap();
        let genesis_tx = &genesis_block.transactions()[0];
        let tx_id = genesis_tx.id();
        
        // Should be able to retrieve it
        let retrieved_tx = blockchain.get_transaction(&tx_id).unwrap();
        assert_eq!(retrieved_tx.id(), tx_id);
        assert!(retrieved_tx.is_coinbase());
    }

    #[test]
    fn test_get_blocks_range() {
        let blockchain = Blockchain::default();
        
        let blocks = blockchain.get_blocks_range(0, 0);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].height(), 0);
        
        // Test empty range
        let empty_blocks = blockchain.get_blocks_range(1, 5);
        assert_eq!(empty_blocks.len(), 0);
    }
}