use crate::types::*;
use crate::transaction::Transaction;
use crate::{BlockchainError, Result};
use blockchain_crypto::{Hash256, MerkleTree, hash::sha256};
use serde::{Deserialize, Serialize};

/// Block header containing metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block version
    pub version: u32,
    /// Hash of the previous block
    pub prev_block_hash: BlockId,
    /// Merkle root of all transactions
    pub merkle_root: Hash256,
    /// Block timestamp
    pub timestamp: Timestamp,
    /// Difficulty target
    pub difficulty: Difficulty,
    /// Mining nonce
    pub nonce: u64,
    /// Block height in the chain
    pub height: BlockHeight,
    /// Number of transactions in the block
    pub tx_count: u32,
    /// Total size of the block in bytes
    pub size: u32,
    /// Chain ID for network identification
    pub chain_id: ChainId,
}

impl BlockHeader{
    /// Create a new block header
    pub fn new(
        prev_block_hash: BlockId,
        merkle_root: Hash256,
        difficulty: Difficulty,
        height: BlockHeight,
        tx_count: u32,
        chain_id: ChainId,
    ) -> Self {
        Self {
            version: 1,
            prev_block_hash,
            merkle_root,
            timestamp: Timestamp::now(),
            difficulty,
            nonce: 0,
            height,
            tx_count,
            size: 0,
            chain_id,
        }
    }


    ///calculate header hash
    pub fn hash(&self) -> Hash256{
        let serialized = bincode::serialize(self)
            .expect("Block header serialization should not fail");
        sha256(&serialized)
    }


    ///get block id from header hash
    pub fn id(%self) -> BlockId {
        BlockId::new(self.hash())
    }


    ///Ceck if block meets difficulty target
    pub fn meets_difficulty(&self) -> bool {
        let hash = self.hash();
        blockchain_crypto::hash;;meets_difficulty(&hash, self.difficulty)
    }


    ///get hash difficulry
    pub fn hash_difficulty(&self) -> u32 {
        blockchain_crypto::hash::hash_difficulty(self.hash())
    }




}



#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockBody{
    ///list of transactions in the block

    pub transactions: Vec<Transaction>,
}


impl BlockBody{
    //create a new block body
    pub new(transactions: Vec<Transaction>) -> Self{
        Self{transactions}
    }


    ///calculate merkle root for all transactions
    pub calculate_merkle_root(&self) -> Result<Hash256> {
        if self.transactions.is_empty() {
            return Ok(Hash256::zero());
        }


        let tx_hashes: Vec<Hash256> = self.transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();

        let merkle_tree = MerkleTree::new(tx_hashes)
            .map_err(|e| BlockchainError::InvalidBlock(
                format!("Merkle tree error: {e}" , e)
                ))

        Ok(merkle_tree.root())
    }


    ///get size of all transactions
    pub fn calculate_size(&self) -> usize {
        self.transactions.iter()
        .map(|tx| tx.size())
        .sum()
    }


    ///get transaction by hash
    pub get_transaction(&self, tx_id: &TxId) -> Option<&Transaction> {
        self.transactions.iter()
            .find(|tx| tx.id() == *tx_id)
    }

    ///check if block contains a transaction
    pub fn contains_transaction(&self, tx_id: &TxId) ->bool {
        self.get_transaction(tx_id).is_some()
    }


    ///Get coinbase transaction (first transaction) 
    pub fn coinbase_transaction(&self) -> Option<&Transaction>{
        self.transactions.first()
            .filter(|tx| tx.is_coinbase())
    }


    ///get all non coinbase transactions
    pub fn regular_transactions(&self) -> Vec<&Transaction> {
        self.transactions.iter()
            .filter(|tx| !tx.is_coinbase())
            .collect()
    }


    ///calculate the total fees from all transactions
    pub fn total_fees(&self) -> Amount {
        self.transactions.iter()
            .map(|tx| tx.calculate_gas_fee())
            .sum()
    }
}



///complete block struture
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    ///block header
    pub header: BlockHeader,

    ///block body with transactions
    pub body: BlockBody,
}


impl Block {
    ///create new block
    pub fn new(
        prev_block_hash: BlockId,
        transactions: Vec<Transaction>,
        difficulty: Difficulty,
        height: BlockHeight,
        chain_id: ChainId,

        ) -> Result<Self> {
        let body = BlockBody::new(
            transactions
            )

        let merkle_root = body.calculate_merkle_root()?;
        let size = body.calculate_size as u32;

        let header = BlockHeader::new(
            prev_block_hash,
            merkle_root,
            difficulty,
            height,
            body.transactions.len() as u32,
            chain_id,
            );

        header.size = size;

        Ok(Self{header, body})
    }


    pub fn genesis(chain_id: ChainId, coinbase_recipient: blockchain_crypto::Adress, reward: Amount) -> Result<Self> {
        let coinbase_tx = Transaction::new_coinbase(coinbase_recipient, reward, 0);

        Self::new(
            BlockId::genesis(),
            vec![coinbase_tx],
            1, //low difficulty for genesis
            0, ///genesis height
            chain_id,
            )
    }


    ///get block hash
    pub fn hash(&self) -> Hash256 {
        self.header.hash()
    }


    ///get block id
    pub fn id(&self) -> BlockId{
        self.header.id()
    }


    ///get block timestamp
    pub fn timestamp(&self) -> Timestamp{
        self.header.timestamp
    }

    ///get number of transactions
    pub fn transaction_count(&self) -> usize{
        self.body.transactions.len()
    }

    //get all transactions
    pub fn transactions(&self) -> &[Transaction] {
        &self.body.transactions
    }


    ///get transaction by id
    pub fn get_transaction(&self, tx_id: &TxId) -> Option<&Transaction> {
        self.body.get_transaction((tx_id))
    }

    ///check if block is genesis block
    pub fn is_genesis(&self) -> bool {
        self.header.height == 0 && self.header.prev_block_hash == BlockId::genesis()
    }



    ///verify block structure and consistency
    // 1. check merkle root matches transactions
    // 2, check transaction count matches header
    // 3, checj that first transaction is coinbase if any transactions exist)
    // 4, check that only first transaction is coinbase.
    // 5. check block size

    pub validate_structure(&self) -> Result<()> {
        //Check merkle root matches transactions
        let calculated_merkle_root = self.body.calculate_merkle_root()?;
        if self.header.merkle_root != calculated_merkle_root {
            return Err(BlockchainError::InvalidBlock(
                "merkle root mismatch".to_string()
                ));
        }


        //check transaction count matches header
        if self.header.tx_count != self.bolde.transactions.len() as u32 {
            return Err(BlockchainError::InvalidBlock(
                "transaction count mismatch".to_string()
                ));
        }

        //check that first trnsaction is coinbase
        if !self.body.transactions.is_empty(){
            let first_tx = &self.body.transactions[0];
            if !first_tx.is_coinbase() {
                return Err(BlockchainError::InvalidBlock(
                    "First transaction must be coinbase".to_string()
                    ));
            }

            //check that only first transaction is coinbase
            for (i, tx) in self.ody.transaactions.iter().enumerate(){
                if i> 0 && tx.is_coinbase() {
                    return Err(BlockchainError::InvalidBlock(
                        "only first transaction can be coinbase".to_string()
                        ))
                }
            }
        }


        //check block size
        let calculated_size = self.body.calculate_size() as u32;
        if self.header.size != calculated_size {
            return Err(BlockchainError::InvalidBlock(
                "Block size mismatch".to_string()
                ));
        }

        Ok(())
    }


    ///mine the block by finding a valid nonce
    pub fn mine(&mut self, max_iterations: Option<u64>) -> Result<bool> {
        let max_iter = max_iterations.unwrap_or(u64::MAX);
        let mut iterations = 0;

        while iterations< max_iter{
            if self.header.meets_difficulty(){
                return Ok(true);

            }

            self.header.nonce = self.self.header.nonce.wrapping_add(1);
            iterations += 1;


            //update timestamp occassionaly to prevent stale work
            if iterations % 1000000 == 0 {
                self.header.timestamp = Timestamp::now();
            }
        }

        //mining timed out
        Ok(false)
    }



    ///get block size in bytes
    pub fn size(&self) -> usize{
        self.header.size as usize
    }

    ///Calculate block reqard (coinbase amount + fees)
    pub fn total_reward(&self) -> Amount{
        let coinbase_reward = self.body.coinbase_transaction()
            .and_then(|tx| tx.amount)
            .unwrap_or(0);

        let fees = self.body.total_fees();

        coinbase_reward + fees
    }
}


/// Block builder for easier construction
pub struct BlockBuilder {
    prev_block_hash: Option<BlockId>,
    transactions: Vec<Transaction>,
    difficulty: Difficulty,
    height: BlockHeight,
    chain_id: ChainId,
    timestamp: Option<Timestamp>,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            prev_block_hash: None,
            transactions: Vec::new(),
            difficulty: 1,
            height: 0,
            chain_id: 1,
            timestamp: None,
        }
    }
    
    pub fn prev_hash(mut self, prev_hash: BlockId) -> Self {
        self.prev_block_hash = Some(prev_hash);
        self
    }
    
    pub fn add_transaction(mut self, transaction: Transaction) -> Self {
        self.transactions.push(transaction);
        self
    }
    
    pub fn transactions(mut self, transactions: Vec<Transaction>) -> Self {
        self.transactions = transactions;
        self
    }
    
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;
        self
    }
    
    pub fn height(mut self, height: BlockHeight) -> Self {
        self.height = height;
        self
    }
    
    pub fn chain_id(mut self, chain_id: ChainId) -> Self {
        self.chain_id = chain_id;
        self
    }
    
    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
    
    pub fn build(self) -> Result<Block> {
        let prev_hash = self.prev_block_hash
            .unwrap_or_else(BlockId::genesis);
        
        let mut block = Block::new(
            prev_hash,
            self.transactions,
            self.difficulty,
            self.height,
            self.chain_id,
        )?;
        
        if let Some(timestamp) = self.timestamp {
            block.header.timestamp = timestamp;
        }
        
        Ok(block)
    }
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}



// ////////////////======tests========\\\\\\\\\\\\\\\\\\\\\\\\ \\


#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_crypto::{signature::generate_keypair, address::public_key_to_address, AddressType};

    #[test]
    fn test_block_creation() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        let coinbase_tx = Transaction::new_coinbase(address, 5000000000, 1);
        let prev_hash = BlockId::new(sha256(b"previous block"));
        
        let block = Block::new(
            prev_hash,
            vec![coinbase_tx],
            20, // difficulty
            1,  // height
            1,  // chain_id
        ).unwrap();
        
        assert_eq!(block.height(), 1);
        assert_eq!(block.prev_hash(), prev_hash);
        assert_eq!(block.transaction_count(), 1);
        assert!(block.transactions()[0].is_coinbase());
    }

    #[test]
    fn test_genesis_block() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        let genesis = Block::genesis(1, address, 5000000000).unwrap();
        
        assert!(genesis.is_genesis());
        assert_eq!(genesis.height(), 0);
        assert_eq!(genesis.prev_hash(), BlockId::genesis());
        assert_eq!(genesis.transaction_count(), 1);
    }

    #[test]
    fn test_block_verification() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        let coinbase_tx = Transaction::new_coinbase(address, 5000000000, 1);
        let prev_hash = BlockId::new(sha256(b"previous block"));
        
        let block = Block::new(
            prev_hash,
            vec![coinbase_tx],
            20,
            1,
            1,
        ).unwrap();
        
        // Block should verify successfully
        assert!(block.verify_structure().is_ok());
    }

    #[test]
    fn test_block_builder() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        let coinbase_tx = Transaction::new_coinbase(address, 5000000000, 1);
        let prev_hash = BlockId::new(sha256(b"previous block"));
        
        let block = BlockBuilder::new()
            .prev_hash(prev_hash)
            .add_transaction(coinbase_tx)
            .difficulty(20)
            .height(1)
            .chain_id(1)
            .build()
            .unwrap();
        
        assert_eq!(block.height(), 1);
        assert_eq!(block.prev_hash(), prev_hash);
        assert_eq!(block.header.difficulty, 20);
    }

    #[test]
    fn test_block_mining() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        let coinbase_tx = Transaction::new_coinbase(address, 5000000000, 1);
        let prev_hash = BlockId::new(sha256(b"previous block"));
        
        let mut block = Block::new(
            prev_hash,
            vec![coinbase_tx],
            1, // Very low difficulty
            1,
            1,
        ).unwrap();
        
        // Should be able to mine with low difficulty
        let result = block.mine(Some(10000));
        assert!(result.is_ok());
        
        if result.unwrap() {
            assert!(block.header.meets_difficulty());
        }
    }

    #[test]
    fn test_merkle_root_calculation() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        let tx1 = Transaction::new_coinbase(address, 5000000000, 1);
        let tx2 = Transaction::new_coinbase(address, 1000000000, 2);
        
        let body = BlockBody::new(vec![tx1, tx2]);
        let merkle_root = body.calculate_merkle_root().unwrap();
        
        assert!(!merkle_root.is_zero());
        
        // Same transactions should produce same merkle root
        let body2 = BlockBody::new(vec![
            Transaction::new_coinbase(address, 5000000000, 1),
            Transaction::new_coinbase(address, 1000000000, 2),
        ]);
        let merkle_root2 = body2.calculate_merkle_root().unwrap();
        
        assert_eq!(merkle_root, merkle_root2);
    }
}