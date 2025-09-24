use crate::types::*;
use crate::transaction::Transaction;
use crate::state::WorldState;
use crate::{BlockchainError, Result};
use blockchain_crypto::Address;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;
use chrono::{DateTime, Utc, Duration};


///Transaction with priority information for mempool ordering
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrioritizedTransaction {
    /// the transaction
    pub transaction: Transaction,
    /// Fee per byte for prioritization
    pub fee_per_byte: u64,
    ///time when transaction was added to mempool
    pub added_time: DateTime<Utc>,
    ///Number of confirmations required
    pub confirmation_needed: u32,
}

impl PrioritizedTransaction{
    pub fn new(transaction: Transaction) -> Self {
        Self{
            transaction,
            fee_per_byte,
            added_time: Utc::now(),
            confirmation_needed: 1,
        }
    }

    pub fn id(&self) -> TxId{
        self.transaction.id()
    }

}


impl PartialOrd for PrioritizedTransaction{
    fn partial_cmp(&self, other: &self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedTransaction {
    fn cmp(&self, other: &self) -> Ordering{
        //Higher fee per byte = higher priority
        self.fee_per_byte.cmp(&other.fee_per_byte)
            .then_with(|| other.added_time.cmp(&self.added_time)) //Earlier = higher priority
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolConfig {
    ///maximum number of transactions in mempool
    pub max_transactions: usize,
    ///maximum memory usage in bytes
    pub max_memory: usize,
    ///maximum age of transactions before eviction
    pub max_age: Duration,
    ///minimum fee per byte to accept
    pub min_fee_per_byte: u64,
    ///maximum transaction size in bytes
    pub max_transaction_size: usize,
}


impl Default for MempoolConfig{
    fn default() -> Self {
        Self{
            max_transactions: 1000,
            max_memory: 100 *1024 * 1024, //100MB
            max_age: Duration::hours(24),
            min_fee_per_byte: 1;
            max_transaction_size: 1024 * 1024 //1MB
        }
    }
}

///transactiion pool(mempool) for pending transactions
#[derive(Debug, Clone)]
pub struct TransactionPool {
    ///Transactions ordered by riority (fee)
    priority_queue: BinaryHeap<PrioritizedTransaction>,
    ///Quick lookup by transactio id
    transactions: HashMap<TxId, PrioritizedTransaction>,
    ///transactions by sender address(for nonce validation)
    by_sender: HashMap<Address, Vec<TxId>>,
    ///set of spent outpoints to prevent double spending
    spent_outpoints: HashSet<OutPoint>,
    ///curren memory usage
    memory_usage: usize,
    //Configuration
    conig: MempoolConfig,
}



impl TransactionPool{
    ///create new transaction pool
    pub fn new(config: MempoolConfig) -> Sel {
        Self {
            priority_queue: BinaryHeap::new(),
            transactions: HashMap::new(),
            by_sender: HashMap::new(),
            spent_outpoints: HashSet::new(),
            memory_usage: 0,
            config,
        }
    }

    pub fn add_transaction(
        &mut self,
        transaction: Transaction,
        world_state: &WorldState,
        ) -> Result<TxId> {
        let tx_id = transaction.id();

        //check if transaction already exists
        if self.transactions.contains_key(&tx_id){
            return Err(BlockchainError::MempoolError(
                "Transaction already in mempool".to_string()
                ));
        }

        //validate transaction
        self.validate_transaction(&transaction, world_state)?;

        //check mempool limits
        self.check_limits(&transaction)?;


        let prioritized_tx = PrioritizedTransaction::new(transaction.clone());

        //check for conflicts(double spending)
        self.check_conflicts(&transaction)?;

        //add spent outpoints to conflict detection
        for input in &transaction.inputs {
            self.spent_outpoints.insert(input.prev_output);
        }

        //update sender tracking
        if let Some(from) = transaction.from {
            self.by_sender.entry(from)
                .or_insert_with(Vec::new)
                .push(tx_id);
        }

        self.memory_usage += transaction.size();


        //add to collections
        self.priority_queue.push(prioritized_tx.clone());
        self.transactions.insert(tx_id, prioritized_tx);


        //evict old transactions if needed
        self.evict_if_needed()?;

        Ok(tx_id)
    }


    ///remove tx from pool

    pub fn remove_transaction(&mut self, tx_id: &TxId) -> Option<Transaction> {
        if let Some(prioritized_tx) = self.transactions.remove(tx_id){
            let transaction = prioritized_tx.transaction;

            //update memory usage
            self.memory_usage = self.memory_usage.saturating_sub(transaction.size());

            //remove spent outpoints
            for input in &transaction.inputs {
                self.spent_outpoints.remove(&input.prev_output);
            }


            //update sender tracking
            if let Some(from) = transaction.from {
                if let Some(sender_txs) = self.by_sender.get_mut(&from) {
                    sender_txs.retain(|id| id != tx_id);

                    if sender_txs.is_empty() {
                        self.by_sender.remove(&from);
                    }
                }
            }

         // Rebuild priority queue (expensive but necessary)
            self.rebuild_priority_queue();
            
            Some(transaction)
        } else {
            None
        }
    }
    
    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &TxId) -> Option<&Transaction> {
        self.transactions.get(tx_id).map(|ptx| &ptx.transaction)
    }
    
    /// Get transactions for block creation (highest priority first)
    pub fn get_transactions_for_block(
        &self, 
        max_count: usize,
        max_size: usize,
        world_state: &WorldState,
    ) -> Vec<Transaction> {
        let mut selected = Vec::new();
        let mut total_size = 0;
        let mut used_outpoints = HashSet::new();
        let mut nonce_tracker: HashMap<Address, Nonce> = HashMap::new();
        
        // Initialize nonce tracker with current world state
        for (address, _) in &self.by_sender {
            nonce_tracker.insert(*address, world_state.get_nonce(address));
        }
        
        // Sort transactions by priority
        let mut sorted_txs: Vec<_> = self.transactions.values().collect();
        sorted_txs.sort_by(|a, b| b.cmp(a)); // Highest priority first
        
        for prioritized_tx in sorted_txs {
            let tx = &prioritized_tx.transaction;
            
            // Check limits
            if selected.len() >= max_count {
                break;
            }
            
            let tx_size = tx.size();
            if total_size + tx_size > max_size {
                continue;
            }
            
            // Check for conflicts with already selected transactions
            let mut has_conflict = false;
            for input in &tx.inputs {
                if used_outpoints.contains(&input.prev_output) {
                    has_conflict = true;
                    break;
                }
            }
            
            if has_conflict {
                continue;
            }
            
            // Check nonce ordering for account-based transactions
            if let (Some(from), Some(tx_nonce)) = (tx.from, tx.nonce) {
                let expected_nonce = nonce_tracker.get(&from).copied().unwrap_or(0);
                if tx_nonce != expected_nonce {
                    continue; // Skip out-of-order transactions
                }
                nonce_tracker.insert(from, expected_nonce + 1);
            }
            
            // Add transaction
            for input in &tx.inputs {
                used_outpoints.insert(input.prev_output);
            }
            
            total_size += tx_size;
            selected.push(tx.clone());
        }
        
        selected
    }
    
    /// Get all transactions
    pub fn get_all_transactions(&self) -> Vec<&Transaction> {
        self.transactions.values()
            .map(|ptx| &ptx.transaction)
            .collect()
    }
    
    /// Get transactions by sender
    pub fn get_transactions_by_sender(&self, sender: &Address) -> Vec<&Transaction> {
        self.by_sender.get(sender)
            .map(|tx_ids| {
                tx_ids.iter()
                    .filter_map(|tx_id| self.get_transaction(tx_id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get pending transaction count
    pub fn len(&self) -> usize {
        self.transactions.len()
    }
    
    /// Check if mempool is empty
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
    
    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }
    
    /// Clear all transactions
    pub fn clear(&mut self) {
        self.priority_queue.clear();
        self.transactions.clear();
        self.by_sender.clear();
        self.spent_outpoints.clear();
        self.memory_usage = 0;
    }
    
    /// Validate transaction before adding to pool
    fn validate_transaction(&self, tx: &Transaction, world_state: &WorldState) -> Result<()> {
        // Check transaction size
        if tx.size() > self.config.max_transaction_size {
            return Err(BlockchainError::MempoolError(
                "Transaction too large".to_string()
            ));
        }
        
        // Check minimum fee
        let fee_per_byte = if tx.size() > 0 {
            tx.calculate_gas_fee() / tx.size() as u64
        } else {
            0
        };
        
        if fee_per_byte < self.config.min_fee_per_byte {
            return Err(BlockchainError::MempoolError(
                format!("Fee too low: {} < {}", fee_per_byte, self.config.min_fee_per_byte)
            ));
        }
        
        // Skip validation for coinbase transactions
        if tx.is_coinbase() {
            return Ok(());
        }
        
        // Validate account-based transaction
        if let (Some(from), Some(tx_nonce)) = (tx.from, tx.nonce) {
            let account = world_state.get_account(&from);
            let balance = account.balance;
            let current_nonce = account.nonce;
            
            // Check balance
            let total_cost = tx.amount.unwrap_or(0) + tx.calculate_gas_fee();
            if balance < total_cost {
                return Err(BlockchainError::InsufficientBalance {
                    required: total_cost,
                    available: balance,
                });
            }
            
            // Check nonce (must be current nonce or higher)
            if tx_nonce < current_nonce {
                return Err(BlockchainError::MempoolError(
                    format!("Nonce too low: {} < {}", tx_nonce, current_nonce)
                ));
            }
        }
        
        // Validate UTXO-based transaction
        for input in &tx.inputs {
            if let Some(utxo) = world_state.utxo_set().get_utxo(&input.prev_output) {
                // Check if UTXO can be spent
                if utxo.is_coinbase && world_state.block_height() - utxo.block_height < 100 {
                    return Err(BlockchainError::MempoolError(
                        "Coinbase UTXO not mature enough".to_string()
                    ));
                }
            } else {
                return Err(BlockchainError::MempoolError(
                    format!("UTXO not found: {}", input.prev_output)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check mempool limits
    fn check_limits(&self, tx: &Transaction) -> Result<()> {
        if self.transactions.len() >= self.config.max_transactions {
            return Err(BlockchainError::MempoolError(
                "Mempool transaction limit reached".to_string()
            ));
        }
        
        if self.memory_usage + tx.size() > self.config.max_memory {
            return Err(BlockchainError::MempoolError(
                "Mempool memory limit reached".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Check for double spending conflicts
    fn check_conflicts(&self, tx: &Transaction) -> Result<()> {
        for input in &tx.inputs {
            if self.spent_outpoints.contains(&input.prev_output) {
                return Err(BlockchainError::DoubleSpending(
                    format!("Outpoint already spent: {}", input.prev_output)
                ));
            }
        }
        Ok(())
    }
    
    /// Evict old or low-priority transactions if needed
    fn evict_if_needed(&mut self) -> Result<()> {
        let now = Utc::now();
        let mut to_remove = Vec::new();
        
        // Find transactions that are too old
        for (tx_id, prioritized_tx) in &self.transactions {
            if now.signed_duration_since(prioritized_tx.added_time) > self.config.max_age {
                to_remove.push(*tx_id);
            }
        }
        
        // Remove old transactions
        for tx_id in to_remove {
            self.remove_transaction(&tx_id);
        }
        
        // If still over limits, remove lowest priority transactions
        while self.transactions.len() > self.config.max_transactions ||
              self.memory_usage > self.config.max_memory {
            
            if let Some(lowest_priority) = self.find_lowest_priority_transaction() {
                self.remove_transaction(&lowest_priority);
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Find lowest priority transaction
    fn find_lowest_priority_transaction(&self) -> Option<TxId> {
        self.transactions.values()
            .min_by(|a, b| a.cmp(b))
            .map(|ptx| ptx.id())
    }
    
    /// Rebuild priority queue (expensive operation)
    fn rebuild_priority_queue(&mut self) {
        self.priority_queue.clear();
        for prioritized_tx in self.transactions.values() {
            self.priority_queue.push(prioritized_tx.clone());
        }
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: MempoolConfig) {
        self.config = config;
        // Trigger eviction with new limits
        let _ = self.evict_if_needed();
    }
    
    /// Get mempool statistics
    pub fn get_stats(&self) -> MempoolStats {
        let total_fees: u64 = self.transactions.values()
            .map(|ptx| ptx.transaction.calculate_gas_fee())
            .sum();
        
        let avg_fee_per_byte = if !self.transactions.is_empty() {
            self.transactions.values()
                .map(|ptx| ptx.fee_per_byte)
                .sum::<u64>() / self.transactions.len() as u64
        } else {
            0
        };
        
        MempoolStats {
            transaction_count: self.transactions.len(),
            memory_usage: self.memory_usage,
            total_fees,
            avg_fee_per_byte,
            oldest_transaction: self.transactions.values()
                .map(|ptx| ptx.added_time)
                .min(),
        }
    }
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new(MempoolConfig::default())
    }
}

/// Mempool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStats {
    pub transaction_count: usize,
    pub memory_usage: usize,
    pub total_fees: u64,
    pub avg_fee_per_byte: u64,
    pub oldest_transaction: Option<DateTime<Utc>>,
}

/// Main mempool interface
#[derive(Debug, Clone)]
pub struct Mempool {
    /// Transaction pool
    pool: TransactionPool,
}

impl Mempool {
    /// Create new mempool
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            pool: TransactionPool::new(config),
        }
    }
    
    /// Add transaction to mempool
    pub fn add_transaction(
        &mut self,
        transaction: Transaction,
        world_state: &WorldState,
    ) -> Result<TxId> {
        self.pool.add_transaction(transaction, world_state)
    }
    
    /// Remove transaction from mempool
    pub fn remove_transaction(&mut self, tx_id: &TxId) -> Option<Transaction> {
        self.pool.remove_transaction(tx_id)
    }
    
    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &TxId) -> Option<&Transaction> {
        self.pool.get_transaction(tx_id)
    }
    
    /// Check if transaction exists in mempool
    pub fn contains_transaction(&self, tx_id: &TxId) -> bool {
        self.pool.get_transaction(tx_id).is_some()
    }
    
    /// Get transactions for block creation
    pub fn get_transactions_for_block(
        &self,
        max_count: usize,
        max_size: usize,
        world_state: &WorldState,
    ) -> Vec<Transaction> {
        self.pool.get_transactions_for_block(max_count, max_size, world_state)
    }
    
    /// Remove multiple transactions (e.g., after block confirmation)
    pub fn remove_transactions(&mut self, tx_ids: &[TxId]) -> Vec<Transaction> {
        tx_ids.iter()
            .filter_map(|tx_id| self.remove_transaction(tx_id))
            .collect()
    }
    
    /// Get all pending transactions
    pub fn get_pending_transactions(&self) -> Vec<&Transaction> {
        self.pool.get_all_transactions()
    }
    
    /// Get transactions by sender address
    pub fn get_transactions_by_sender(&self, sender: &Address) -> Vec<&Transaction> {
        self.pool.get_transactions_by_sender(sender)
    }
    
    /// Get mempool size
    pub fn len(&self) -> usize {
        self.pool.len()
    }
    
    /// Check if mempool is empty
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
    
    /// Clear all transactions
    pub fn clear(&mut self) {
        self.pool.clear();
    }
    
    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.pool.memory_usage()
    }
    
    /// Get mempool statistics
    pub fn get_stats(&self) -> MempoolStats {
        self.pool.get_stats()
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: MempoolConfig) {
        self.pool.update_config(config);
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new(MempoolConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_crypto::{signature::generate_keypair, address::public_key_to_address, AddressType};
    use crate::state::{WorldState, AccountState};
    use crate::types::AccountModel;

    #[test]
    fn test_mempool_add_transaction() {
        let mut mempool = Mempool::default();
        let mut world_state = WorldState::new(AccountModel::Account);
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        // Set up world state
        world_state.set_account(addr1, AccountState::new(1000));
        
        // Create transaction
        let tx = Transaction::new_account(addr1, addr2, 100, 0, 21000, 20, vec![]);
        let tx_id = tx.id();
        
        // Add to mempool
        let result = mempool.add_transaction(tx, &world_state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tx_id);
        assert_eq!(mempool.len(), 1);
        assert!(mempool.contains_transaction(&tx_id));
    }

    #[test]
    fn test_mempool_insufficient_balance() {
        let mut mempool = Mempool::default();
        let mut world_state = WorldState::new(AccountModel::Account);
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        // Set up world state with insufficient balance
        world_state.set_account(addr1, AccountState::new(50));
        
        // Create transaction requiring more than available
        let tx = Transaction::new_account(addr1, addr2, 100, 0, 21000, 20, vec![]);
        
        // Should fail
        let result = mempool.add_transaction(tx, &world_state);
        assert!(matches!(result, Err(BlockchainError::InsufficientBalance { .. })));
    }

    #[test]
    fn test_mempool_transaction_selection() {
        let mut mempool = Mempool::default();
        let mut world_state = WorldState::new(AccountModel::Account);
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        // Set up world state
        world_state.set_account(addr1, AccountState::new(10000));
        
        // Create transactions with different fees
        let tx1 = Transaction::new_account(addr1, addr2, 100, 0, 21000, 10, vec![]); // Low fee
        let tx2 = Transaction::new_account(addr1, addr2, 100, 1, 21000, 50, vec![]); // High fee
        let tx3 = Transaction::new_account(addr1, addr2, 100, 2, 21000, 30, vec![]); // Medium fee
        
        // Add to mempool
        mempool.add_transaction(tx1, &world_state).unwrap();
        mempool.add_transaction(tx2, &world_state).unwrap();
        mempool.add_transaction(tx3, &world_state).unwrap();
        
        // Get transactions for block (should be ordered by fee, then nonce)
        let selected = mempool.get_transactions_for_block(10, 1000000, &world_state);
        
        // Should select in nonce order (0, 1, 2) despite fee differences
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].nonce, Some(0));
        assert_eq!(selected[1].nonce, Some(1));
        assert_eq!(selected[2].nonce, Some(2));
    }

    #[test]
    fn test_mempool_remove_transaction() {
        let mut mempool = Mempool::default();
        let mut world_state = WorldState::new(AccountModel::Account);
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        world_state.set_account(addr1, AccountState::new(1000));
        
        let tx = Transaction::new_account(addr1, addr2, 100, 0, 21000, 20, vec![]);
        let tx_id = tx.id();
        
        // Add and remove
        mempool.add_transaction(tx, &world_state).unwrap();
        assert_eq!(mempool.len(), 1);
        
        let removed = mempool.remove_transaction(&tx_id);
        assert!(removed.is_some());
        assert_eq!(mempool.len(), 0);
        assert!(!mempool.contains_transaction(&tx_id));
    }

    #[test]
    fn test_mempool_stats() {
        let mut mempool = Mempool::default();
        let mut world_state = WorldState::new(AccountModel::Account);
        
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        world_state.set_account(addr1, AccountState::new(10000));
        
        // Add some transactions
        for i in 0..3 {
            let tx = Transaction::new_account(addr1, addr2, 100, i, 21000, 20, vec![]);
            mempool.add_transaction(tx, &world_state).unwrap();
        }
        
        let stats = mempool.get_stats();
        assert_eq!(stats.transaction_count, 3);
        assert!(stats.total_fees > 0);
        assert!(stats.memory_usage > 0);
        assert!(stats.oldest_transaction.is_some());
    }
}


        }
    }
}