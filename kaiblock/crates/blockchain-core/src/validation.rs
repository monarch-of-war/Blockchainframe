// use crate::types::*;
// use crate::transaction::Transaction;
// use crate::block::Block;
// use crate::state::WorldState;
// use crate::{BlockchainError, Result};
// use blockchain_crypto::Hash256;
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;


// //validation configuration
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ValidationRules {
// 	//max block size
// 	pub max_block_size: usize,
// 	//max txs per block
// 	pub max_transactions_per_block: usize,
// 	//max transaction size
// 	pub max_transaction_size: usize,
// 	//min transaction fee
// 	pub min_transaction_fee: Fee,
// 	//coinbase maturity period(blocks)
// 	pub coinbase_maturity: BlockHeight,
// 	//max block time drift (seconds)
// 	pub max_block_time_drift: i64,
// 	//dificulty adjastment period(blocks)
// 	pub difficulty_adjustment_period: BlockHeight,
// 	//target block time
// 	pub target_block_time: u64,
// 	//max difficulty adjustment ratio
// 	pub max_difficulty_adjustment: f64,
// 	//enable signature verification
// 	pub verify_signatures: bool,
// 	// '' merkle root verification
// 	pub verify_merkle_root: bool,
// 	// '' double spend checking
// 	pub check_double_spend: bool,

// }


// impl Default for ValidationRules {
//     fn default() -> Self {
//         Self {
//             max_block_size: 2 * 1024 * 1024, // 2MB
//             max_transactions_per_block: 10000,
//             max_transaction_size: 1024 * 1024, // 1MB
//             min_transaction_fee: 1000, // 1000 satoshis
//             coinbase_maturity: 100, // 100 blocks
//             max_block_time_drift: 7200, // 2 hours
//             difficulty_adjustment_period: 2016, // Bitcoin-style
//             target_block_time: 600, // 10 minutes
//             max_difficulty_adjustment: 4.0, // Maximum 4x adjustment
//             verify_signatures: true,
//             verify_merkle_root: true,
//             check_double_spend: true,
//         }
//     }
// }

// /// Block validation context
// #[derive(Debug)]
// pub struct BlockValidationContext<'a> {
//     pub block: &'a Block,
//     pub prev_block: Option<&'a Block>,
//     pub world_state: &'a WorldState,
//     pub rules: &'a ValidationRules,
// }


// //main validator for blockchain components
// #[derive(debug)]
// pub struct Validator {
// 	rules: ValidationRules,
// }


// impl Validator{
// 	//create a new validator with rules
// 	pub new(rules: ValidationRules) -> Self {
// 		Self{rules}
// 	}

// 	//validate a single transaction
// 	pub validate_transaction(
// 		&self,
// 		ctx: TransactionValidationContext,
// 		)-> Result<()> {
// 		let tx = ctx.transaction;

// 		//Basic structure validation
// 	}
// }



///////////////Claudie direct //////////////////////
use crate::types::*;
use crate::transaction::Transaction;
use crate::block::Block;
use crate::state::WorldState;
use crate::{BlockchainError, Result};
use blockchain_crypto::Hash256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Maximum block size in bytes
    pub max_block_size: usize,
    /// Maximum number of transactions per block
    pub max_transactions_per_block: usize,
    /// Maximum transaction size in bytes
    pub max_transaction_size: usize,
    /// Minimum transaction fee
    pub min_transaction_fee: Fee,
    /// Coinbase maturity period (blocks)
    pub coinbase_maturity: BlockHeight,
    /// Maximum block time drift (seconds)
    pub max_block_time_drift: i64,
    /// Difficulty adjustment period (blocks)
    pub difficulty_adjustment_period: BlockHeight,
    /// Target block time (seconds)
    pub target_block_time: u64,
    /// Maximum difficulty adjustment ratio
    pub max_difficulty_adjustment: f64,
    /// Enable signature verification
    pub verify_signatures: bool,
    /// Enable merkle root verification
    pub verify_merkle_root: bool,
    /// Enable double spend checking
    pub check_double_spend: bool,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            max_block_size: 2 * 1024 * 1024, // 2MB
            max_transactions_per_block: 10000,
            max_transaction_size: 1024 * 1024, // 1MB
            min_transaction_fee: 1000, // 1000 satoshis
            coinbase_maturity: 100, // 100 blocks
            max_block_time_drift: 7200, // 2 hours
            difficulty_adjustment_period: 2016, // Bitcoin-style
            target_block_time: 600, // 10 minutes
            max_difficulty_adjustment: 4.0, // Maximum 4x adjustment
            verify_signatures: true,
            verify_merkle_root: true,
            check_double_spend: true,
        }
    }
}

/// Transaction validation context
#[derive(Debug)]
pub struct TransactionValidationContext<'a> {
    pub transaction: &'a Transaction,
    pub world_state: &'a WorldState,
    pub block_height: BlockHeight,
    pub block_timestamp: Timestamp,
    pub rules: &'a ValidationRules,
}

/// Block validation context
#[derive(Debug)]
pub struct BlockValidationContext<'a> {
    pub block: &'a Block,
    pub prev_block: Option<&'a Block>,
    pub world_state: &'a WorldState,
    pub rules: &'a ValidationRules,
}

/// Main validator for blockchain components
#[derive(Debug)]
pub struct Validator {
    rules: ValidationRules,
}

impl Validator {
    /// Create new validator with rules
    pub fn new(rules: ValidationRules) -> Self {
        Self { rules }
    }
    
    /// Validate a single transaction
    pub fn validate_transaction(
        &self,
        ctx: TransactionValidationContext,
    ) -> Result<()> {
        let tx = ctx.transaction;
        
        // Basic structure validation
        self.validate_transaction_structure(tx)?;
        
        // Skip further validation for coinbase transactions
        if tx.is_coinbase() {
            return self.validate_coinbase_transaction(ctx);
        }
        
        // Validate transaction amounts
        self.validate_transaction_amounts(ctx)?;
        
        // Validate signatures if enabled
        if self.rules.verify_signatures {
            self.validate_transaction_signatures(ctx)?;
        }
        
        // Validate account-based transaction
        if tx.from.is_some() {
            self.validate_account_transaction(ctx)?;
        }
        
        // Validate UTXO-based transaction
        if !tx.inputs.is_empty() {
            self.validate_utxo_transaction(ctx)?;
        }
        
        // Validate transaction fees
        self.validate_transaction_fees(ctx)?;
        
        // Validate time locks
        self.validate_time_locks(ctx)?;
        
        Ok(())
    }
    
    /// Validate block structure and transactions
    pub fn validate_block(
        &self,
        ctx: BlockValidationContext,
    ) -> Result<()> {
        let block = ctx.block;
        
        // Basic structure validation
        self.validate_block_structure(ctx)?;
        
        // Validate block header
        self.validate_block_header(ctx)?;
        
        // Validate block size
        self.validate_block_size(ctx)?;
        
        // Validate timestamp
        self.validate_block_timestamp(ctx)?;
        
        // Validate difficulty
        self.validate_block_difficulty(ctx)?;
        
        // Validate merkle root if enabled
        if self.rules.verify_merkle_root {
            self.validate_merkle_root(ctx)?;
        }
        
        // Validate all transactions in block
        self.validate_block_transactions(ctx)?;
        
        Ok(())
    }
    
    /// Validate transaction structure
    fn validate_transaction_structure(&self, tx: &Transaction) -> Result<()> {
        // Check version
        if tx.version == 0 {
            return Err(BlockchainError::InvalidTransaction(
                "Invalid transaction version".to_string()
            ));
        }
        
        // Check size
        if tx.size() > self.rules.max_transaction_size {
            return Err(BlockchainError::InvalidTransaction(
                format!("Transaction too large: {} > {}", tx.size(), self.rules.max_transaction_size)
            ));
        }
        
        // Check that transaction has either inputs/outputs or from/to
        if tx.inputs.is_empty() && tx.outputs.is_empty() && 
           tx.from.is_none() && tx.to.is_none() && !tx.is_coinbase() {
            return Err(BlockchainError::InvalidTransaction(
                "Transaction has no inputs, outputs, or addresses".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate coinbase transaction
    fn validate_coinbase_transaction(
        &self,
        ctx: TransactionValidationContext,
    ) -> Result<()> {
        let tx = ctx.transaction;
        
        // Coinbase must have no inputs
        if !tx.inputs.is_empty() {
            return Err(BlockchainError::InvalidTransaction(
                "Coinbase transaction cannot have inputs".to_string()
            ));
        }
        
        // Coinbase must have at least one output
        if tx.outputs.is_empty() && tx.to.is_none() {
            return Err(BlockchainError::InvalidTransaction(
                "Coinbase transaction must have outputs".to_string()
            ));
        }
        
        // Validate coinbase reward (this would need more context in real implementation)
        // For now, just check that amounts are reasonable
        let total_output = tx.total_output_amount()?;
        if total_output == 0 {
            return Err(BlockchainError::InvalidTransaction(
                "Coinbase transaction must have non-zero output".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate transaction amounts
    fn validate_transaction_amounts(
        &self,
        ctx: TransactionValidationContext,
    ) -> Result<()> {
        let tx = ctx.transaction;
        
        // Check for overflow in outputs
        let _total_output = tx.total_output_amount()?;
        
        // For UTXO transactions, validate input/output balance
        if !tx.inputs.is_empty() {
            let total_input = tx.total_input_amount(&ctx.world_state.utxo_set().utxos)?;
            let total_output = tx.total_output_amount()?;
            
            if total_input < total_output + tx.fee {
                return Err(BlockchainError::InvalidTransaction(
                    format!("Insufficient input amount: {} < {} + {}", 
                           total_input, total_output, tx.fee)
                ));
            }
        }
        
        // Check for zero or negative amounts
        for output in &tx.outputs {
            if output.amount == 0 {
                return Err(BlockchainError::InvalidTransaction(
                    "Transaction output cannot be zero".to_string()
                ));
            }
        }
        
        if let Some(amount) = tx.amount {
            if amount == 0 {
                return Err(BlockchainError::InvalidTransaction(
                    "Transaction amount cannot be zero".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate transaction signatures
    fn validate_transaction_signatures(
        &self,
        ctx: TransactionValidationContext,
    ) -> Result<()> {
        let tx = ctx.transaction;
        
        // Validate UTXO input signatures
        if !tx.verify_signatures(&ctx.world_state.utxo_set().utxos)? {
            return Err(BlockchainError::InvalidTransaction(
                "Invalid transaction signature".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate account-based transaction
    fn validate_account_transaction(
        &self,
        ctx: TransactionValidationContext,
    ) -> Result<()> {
        let tx = ctx.transaction;
        let from = tx.from.ok_or_else(|| {
            BlockchainError::InvalidTransaction("Missing sender address".to_string())
        })?;
        
        let account = ctx.world_state.get_account(&from);
        
        // Validate nonce
        if let Some(tx_nonce) = tx.nonce {
            if tx_nonce != account.nonce {
                return Err(BlockchainError::InvalidTransaction(
                    format!("Invalid nonce: expected {}, got {}", account.nonce, tx_nonce)
                ));
            }
        }
        
        // Validate balance
        let total_cost = tx.amount.unwrap_or(0) + tx.calculate_gas_fee();
        if account.balance < total_cost {
            return Err(BlockchainError::InsufficientBalance {
                required: total_cost,
                available: account.balance,
            });
        }
        
        // Validate gas limits
        if let Some(gas_limit) = tx.gas_limit {
            if gas_limit == 0 {
                return Err(BlockchainError::InvalidTransaction(
                    "Gas limit cannot be zero".to_string()
                ));
            }
            
            // Could add maximum gas limit check here
            if gas_limit > 10_000_000 { // Example limit
                return Err(BlockchainError::InvalidTransaction(
                    "Gas limit too high".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate UTXO-based transaction
    fn validate_utxo_transaction(
        &self,
        ctx: TransactionValidationContext,
    ) -> Result<()> {
        let tx = ctx.transaction;
        let utxo_set = ctx.world_state.utxo_set();
        
        // Validate that all inputs exist and are unspent
        for input in &tx.inputs {
            let utxo = utxo_set.get_utxo(&input.prev_output)
                .ok_or_else(|| BlockchainError::InvalidTransaction(
                    format!("UTXO not found: {}", input.prev_output)
                ))?;
            
            // Check coinbase maturity
            if utxo.is_coinbase && 
               ctx.block_height - utxo.block_height < self.rules.coinbase_maturity {
                return Err(BlockchainError::InvalidTransaction(
                    format!("Coinbase UTXO not mature: {} < {}", 
                           ctx.block_height - utxo.block_height, 
                           self.rules.coinbase_maturity)
                ));
            }
        }
        
        // Check for double spending within transaction
        let mut used_outpoints = std::collections::HashSet::new();
        for input in &tx.inputs {
            if !used_outpoints.insert(input.prev_output) {
                return Err(BlockchainError::DoubleSpending(
                    format!("Double spend within transaction: {}", input.prev_output)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate transaction fees
    fn validate_transaction_fees(
        &self,
        ctx: TransactionValidationContext,
    ) -> Result<()> {
        let tx = ctx.transaction;
        let fee = tx.calculate_gas_fee();
        
        if fee < self.rules.min_transaction_fee {
            return Err(BlockchainError::InvalidTransaction(
                format!("Transaction fee too low: {} < {}", fee, self.rules.min_transaction_fee)
            ));
        }
        
        Ok(())
    }
    
    /// Validate time locks
    fn validate_time_locks(
        &self,
        ctx: TransactionValidationContext,
    ) -> Result<()> {
        let tx = ctx.transaction;
        
        // Check lock time
        if tx.lock_time > 0 {
            // Lock time can be either block height or timestamp
            if tx.lock_time < 500_000_000 {
                // Interpreted as block height
                if ctx.block_height < tx.lock_time as BlockHeight {
                    return Err(BlockchainError::InvalidTransaction(
                        format!("Transaction locked until block {}", tx.lock_time)
                    ));
                }
            } else {
                // Interpreted as timestamp
                if ctx.block_timestamp.to_unix_timestamp() < tx.lock_time as i64 {
                    return Err(BlockchainError::InvalidTransaction(
                        format!("Transaction locked until timestamp {}", tx.lock_time)
                    ));
                }
            }
        }
        
        // Check sequence numbers for relative time locks
        for input in &tx.inputs {
            if input.sequence < 0xfffffffe {
                // Sequence number indicates relative lock time
                // Implementation would depend on specific BIP-68 rules
                // For now, just ensure sequence is valid
                if input.sequence == 0 {
                    return Err(BlockchainError::InvalidTransaction(
                        "Invalid sequence number".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate block structure
    fn validate_block_structure(&self, ctx: BlockValidationContext) -> Result<()> {
        let block = ctx.block;
        
        // Validate that block has transactions
        if block.transactions().is_empty() {
            return Err(BlockchainError::InvalidBlock(
                "Block must contain at least one transaction (coinbase)".to_string()
            ));
        }
        
        // Validate that first transaction is coinbase
        if !block.transactions()[0].is_coinbase() {
            return Err(BlockchainError::InvalidBlock(
                "First transaction must be coinbase".to_string()
            ));
        }
        
        // Validate that only first transaction is coinbase
        for (i, tx) in block.transactions().iter().enumerate() {
            if i > 0 && tx.is_coinbase() {
                return Err(BlockchainError::InvalidBlock(
                    "Only first transaction can be coinbase".to_string()
                ));
            }
        }
        
        // Validate transaction count
        if block.transaction_count() > self.rules.max_transactions_per_block {
            return Err(BlockchainError::InvalidBlock(
                format!("Too many transactions: {} > {}", 
                       block.transaction_count(), 
                       self.rules.max_transactions_per_block)
            ));
        }
        
        Ok(())
    }
    
    /// Validate block header
    fn validate_block_header(&self, ctx: BlockValidationContext) -> Result<()> {
        let header = &ctx.block.header;
        
        // Validate version
        if header.version == 0 {
            return Err(BlockchainError::InvalidBlock(
                "Invalid block version".to_string()
            ));
        }
        
        // Validate transaction count matches header
        if header.tx_count != ctx.block.transaction_count() as u32 {
            return Err(BlockchainError::InvalidBlock(
                "Transaction count mismatch".to_string()
            ));
        }
        
        // Validate size matches header
        if header.size != ctx.block.size() as u32 {
            return Err(BlockchainError::InvalidBlock(
                "Block size mismatch".to_string()
            ));
        }
        
        // Validate height sequence
        if let Some(prev_block) = ctx.prev_block {
            if header.height != prev_block.height() + 1 {
                return Err(BlockchainError::InvalidBlock(
                    format!("Invalid block height: expected {}, got {}", 
                           prev_block.height() + 1, header.height)
                ));
            }
            
            // Validate previous block hash
            if header.prev_block_hash != prev_block.id() {
                return Err(BlockchainError::InvalidBlock(
                    "Previous block hash mismatch".to_string()
                ));
            }
        } else if header.height != 0 {
            // Genesis block validation
            return Err(BlockchainError::InvalidBlock(
                "Genesis block must have height 0".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate block size
    fn validate_block_size(&self, ctx: BlockValidationContext) -> Result<()> {
        let block_size = ctx.block.size();
        
        if block_size > self.rules.max_block_size {
            return Err(BlockchainError::InvalidBlock(
                format!("Block too large: {} > {}", block_size, self.rules.max_block_size)
            ));
        }
        
        Ok(())
    }
    
    /// Validate block timestamp
    fn validate_block_timestamp(&self, ctx: BlockValidationContext) -> Result<()> {
        let block_timestamp = ctx.block.timestamp().to_unix_timestamp();
        let current_time = chrono::Utc::now().timestamp();
        
        // Check that block timestamp is not too far in the future
        if block_timestamp > current_time + self.rules.max_block_time_drift {
            return Err(BlockchainError::InvalidBlock(
                format!("Block timestamp too far in future: {} > {}", 
                       block_timestamp, current_time + self.rules.max_block_time_drift)
            ));
        }
        
        // Check that block timestamp is after previous block
        if let Some(prev_block) = ctx.prev_block {
            let prev_timestamp = prev_block.timestamp().to_unix_timestamp();
            if block_timestamp <= prev_timestamp {
                return Err(BlockchainError::InvalidBlock(
                    "Block timestamp must be greater than previous block".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate block difficulty and proof of work
    fn validate_block_difficulty(&self, ctx: BlockValidationContext) -> Result<()> {
        let header = &ctx.block.header;
        
        // Check that block meets difficulty target
        if !header.meets_difficulty() {
            return Err(BlockchainError::InvalidBlock(
                format!("Block does not meet difficulty target: {} < {}", 
                       header.hash_difficulty(), header.difficulty)
            ));
        }
        
        // Validate difficulty adjustment (simplified)
        if let Some(prev_block) = ctx.prev_block {
            let expected_difficulty = self.calculate_next_difficulty(ctx.prev_block, ctx.block.height())?;
            
            // Allow some tolerance for difficulty adjustments
            let min_difficulty = (expected_difficulty as f64 / self.rules.max_difficulty_adjustment) as u32;
            let max_difficulty = (expected_difficulty as f64 * self.rules.max_difficulty_adjustment) as u32;
            
            if header.difficulty < min_difficulty || header.difficulty > max_difficulty {
                return Err(BlockchainError::InvalidBlock(
                    format!("Invalid difficulty adjustment: {} not in range [{}, {}]", 
                           header.difficulty, min_difficulty, max_difficulty)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Calculate next difficulty (simplified implementation)
    fn calculate_next_difficulty(
        &self, 
        _prev_block: Option<&Block>, 
        current_height: BlockHeight
    ) -> Result<Difficulty> {
        // Simplified difficulty calculation
        // In a real implementation, this would look at block times over the adjustment period
        
        if current_height % self.rules.difficulty_adjustment_period == 0 {
            // Difficulty adjustment block
            // For now, just return a constant difficulty
            Ok(1)
        } else {
            // No adjustment needed
            Ok(1)
        }
    }
    
    /// Validate merkle root
    fn validate_merkle_root(&self, ctx: BlockValidationContext) -> Result<()> {
        let calculated_root = ctx.block.body.calculate_merkle_root()?;
        
        if calculated_root != ctx.block.header.merkle_root {
            return Err(BlockchainError::InvalidBlock(
                "Merkle root mismatch".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate all transactions in block
    fn validate_block_transactions(&self, ctx: BlockValidationContext) -> Result<()> {
        let block_height = ctx.block.height();
        let block_timestamp = ctx.block.timestamp();
        
        // Track double spending within the block
        let mut used_outpoints = std::collections::HashSet::new();
        
        for (i, tx) in ctx.block.transactions().iter().enumerate() {
            // Create transaction validation context
            let tx_ctx = TransactionValidationContext {
                transaction: tx,
                world_state: ctx.world_state,
                block_height,
                block_timestamp,
                rules: ctx.rules,
            };
            
            // Validate individual transaction
            self.validate_transaction(tx_ctx)?;
            
            // Check for double spending within block
            if self.rules.check_double_spend {
                for input in &tx.inputs {
                    if !used_outpoints.insert(input.prev_output) {
                        return Err(BlockchainError::InvalidBlock(
                            format!("Double spend in block at transaction {}: {}", 
                                   i, input.prev_output)
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get validation rules
    pub fn rules(&self) -> &ValidationRules {
        &self.rules
    }
    
    /// Update validation rules
    pub fn update_rules(&mut self, rules: ValidationRules) {
        self.rules = rules;
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new(ValidationRules::default())
    }
}

/// Batch validation for multiple transactions
pub fn validate_transactions_batch(
    validator: &Validator,
    transactions: &[Transaction],
    world_state: &WorldState,
    block_height: BlockHeight,
    block_timestamp: Timestamp,
) -> Result<Vec<Result<()>>> {
    let results: Vec<Result<()>> = transactions.iter()
        .map(|tx| {
            let ctx = TransactionValidationContext {
                transaction: tx,
                world_state,
                block_height,
                block_timestamp,
                rules: validator.rules(),
            };
            validator.validate_transaction(ctx)
        })
        .collect();
    
    Ok(results)
}

/// Validate chain consistency
pub fn validate_chain_consistency(
    validator: &Validator,
    blocks: &[Block],
    initial_state: &WorldState,
) -> Result<()> {
    if blocks.is_empty() {
        return Ok(());
    }
    
    let mut current_state = initial_state.clone();
    let mut prev_block: Option<&Block> = None;
    
    for block in blocks {
        // Validate block
        let ctx = BlockValidationContext {
            block,
            prev_block,
            world_state: &current_state,
            rules: validator.rules(),
        };
        
        validator.validate_block(ctx)?;
        
        // Apply block to state for next validation
        for tx in block.transactions() {
            current_state.apply_transaction(tx)?;
        }
        current_state.set_block_height(block.height());
        
        prev_block = Some(block);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_crypto::{signature::generate_keypair, address::public_key_to_address, AddressType};
    use crate::state::{WorldState, AccountState};
    use crate::types::AccountModel;

    #[test]
    fn test_transaction_validation() {
        let validator = Validator::default();
        let mut world_state = WorldState::new(AccountModel::Account);
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        // Set up world state
        world_state.set_account(addr1, AccountState::new(10000));
        
        // Create valid transaction
        let tx = Transaction::new_account(addr1, addr2, 1000, 0, 21000, 20, vec![]);
        
        let ctx = TransactionValidationContext {
            transaction: &tx,
            world_state: &world_state,
            block_height: 1,
            block_timestamp: Timestamp::now(),
            rules: validator.rules(),
        };
        
        // Should validate successfully
        assert!(validator.validate_transaction(ctx).is_ok());
    }

    #[test]
    fn test_transaction_validation_insufficient_balance() {
        let validator = Validator::default();
        let mut world_state = WorldState::new(AccountModel::Account);
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        // Set up world state with insufficient balance
        world_state.set_account(addr1, AccountState::new(100));
        
        // Create transaction requiring more than available
        let tx = Transaction::new_account(addr1, addr2, 1000, 0, 21000, 20, vec![]);
        
        let ctx = TransactionValidationContext {
            transaction: &tx,
            world_state: &world_state,
            block_height: 1,
            block_timestamp: Timestamp::now(),
            rules: validator.rules(),
        };
        
        // Should fail validation
        assert!(matches!(
            validator.validate_transaction(ctx),
            Err(BlockchainError::InsufficientBalance { .. })
        ));
    }

    #[test]
    fn test_block_validation() {
        let validator = Validator::default();
        let mut world_state = WorldState::new(AccountModel::Account);
        
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        // Create a valid block
        let coinbase_tx = Transaction::new_coinbase(address, 5000000000, 1);
        let block = Block::new(
            BlockId::genesis(),
            vec![coinbase_tx],
            1, // Low difficulty
            1,
            1,
        ).unwrap();
        
        let ctx = BlockValidationContext {
            block: &block,
            prev_block: None,
            world_state: &world_state,
            rules: validator.rules(),
        };
        
        // Should validate successfully
        assert!(validator.validate_block(ctx).is_ok());
    }

    #[test]
    fn test_block_validation_no_coinbase() {
        let validator = Validator::default();
        let world_state = WorldState::new(AccountModel::Account);
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        // Create block without coinbase transaction
        let regular_tx = Transaction::new_account(addr1, addr2, 1000, 0, 21000, 20, vec![]);
        let block = Block::new(
            BlockId::genesis(),
            vec![regular_tx],
            1,
            1,
            1,
        ).unwrap();
        
        let ctx = BlockValidationContext {
            block: &block,
            prev_block: None,
            world_state: &world_state,
            rules: validator.rules(),
        };
        
        // Should fail validation
        assert!(validator.validate_block(ctx).is_err());
    }
}