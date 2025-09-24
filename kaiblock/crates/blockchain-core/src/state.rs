use crate::types::*;
use crate::transaction::{Transaction, UTXO};
use crate::{BlockchainError, Result};
use blockchain_crypto::{Hash256, Address, hash::sha256};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use indexmap::IndexMap;


/// Account state for account-based model (like Ethereum)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub AccountState {
    ///account balance
    pub balance: Amount,
    ///account nonce(transaction counter)
    pub nonce: Nonce,
    ///strorage root hash for smart contracts
    pub storage_root: Hash256,
    ///code hash for smrt contracts
    pub code_hash: Hash256,
    ///additional metadata
    pub metadata: HashMap<String, Vec<u8>,
}


impl AccountState{
    ///create new account state
    pub fn new(balance: Amount) -> Self {
        Self {
            balance,
            nonce: 0,
            storage_root: Hash256::zero(),
            code_hash: Hash256::zero(),
            metadata: HashMap::new(),
        }
    }

    ///create empty account
    pub fn empty() -> Self {
        Self;;new(0)
    }


    ///Check if account is empty
    pub is_empty(&self) -> bool{
        self.balance ==0 &&
        self.nonce == 0 &&
        self.storage_root.is_zero() &&
        self.code_hash.is_zero()
    }


    pub fn add_balance(&mut self, amount: Amount) -> Result<()> {
        self.balance = self.balance.checked_add(amount)
            .ok_or_else(|| BlockchainError::InsufficientBalance{
                required: amount,
                available: self.balance,
            });

            Ok(())
    }

    ///subtract balance from account
    pub fn sub_balance(&mut self, amount: Amount) -> Result<()> {
        if self.balance < amount {
            return Err(BlockchainError::InsufficientBalance{
                required: amount,
                available: self.balance
            });
        }
    }
}



/////direct copy from Claudie=========================

/// UTXO set for UTXO-based model (like Bitcoin)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOSet {
    /// Map of outpoints to UTXOs
    utxos: HashMap<OutPoint, UTXO>,
    /// Index by address for efficient lookups
    address_index: HashMap<Address, HashSet<OutPoint>>,
    /// Total value in UTXO set
    total_value: Amount,
}

impl UTXOSet {
    /// Create new empty UTXO set
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
            address_index: HashMap::new(),
            total_value: 0,
        }
    }
    
    /// Add UTXO to the set
    pub fn add_utxo(&mut self, outpoint: OutPoint, utxo: UTXO) -> Result<()> {
        // Check if UTXO already exists
        if self.utxos.contains_key(&outpoint) {
            return Err(BlockchainError::StateError(
                format!("UTXO already exists: {}", outpoint)
            ));
        }
        
        // Update address index
        self.address_index
            .entry(utxo.output.address)
            .or_insert_with(HashSet::new)
            .insert(outpoint);
        
        // Update total value
        self.total_value = self.total_value.checked_add(utxo.output.amount)
            .ok_or_else(|| BlockchainError::StateError("Total value overflow".to_string()))?;
        
        self.utxos.insert(outpoint, utxo);
        Ok(())
    }
    
    /// Remove UTXO from the set
    pub fn remove_utxo(&mut self, outpoint: &OutPoint) -> Result<UTXO> {
        let utxo = self.utxos.remove(outpoint)
            .ok_or_else(|| BlockchainError::StateError(
                format!("UTXO not found: {}", outpoint)
            ))?;
        
        // Update address index
        if let Some(address_utxos) = self.address_index.get_mut(&utxo.output.address) {
            address_utxos.remove(outpoint);
            if address_utxos.is_empty() {
                self.address_index.remove(&utxo.output.address);
            }
        }
        
        // Update total value
        self.total_value -= utxo.output.amount;
        
        Ok(utxo)
    }
    
    /// Get UTXO by outpoint
    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<&UTXO> {
        self.utxos.get(outpoint)
    }
    
    /// Get all UTXOs for an address
    pub fn get_utxos_by_address(&self, address: &Address) -> Vec<(&OutPoint, &UTXO)> {
        self.address_index.get(address)
            .map(|outpoints| {
                outpoints.iter()
                    .filter_map(|outpoint| {
                        self.utxos.get(outpoint)
                            .map(|utxo| (outpoint, utxo))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get balance for an address
    pub fn get_balance(&self, address: &Address) -> Amount {
        self.get_utxos_by_address(address)
            .iter()
            .map(|(_, utxo)| utxo.output.amount)
            .sum()
    }
    
    /// Check if outpoint exists
    pub fn contains(&self, outpoint: &OutPoint) -> bool {
        self.utxos.contains_key(outpoint)
    }
    
    /// Get total number of UTXOs
    pub fn len(&self) -> usize {
        self.utxos.len()
    }
    
    /// Check if UTXO set is empty
    pub fn is_empty(&self) -> bool {
        self.utxos.is_empty()
    }
    
    /// Get total value in UTXO set
    pub fn total_value(&self) -> Amount {
        self.total_value
    }
    
    /// Apply transaction to UTXO set
    pub fn apply_transaction(&mut self, tx: &Transaction, block_height: BlockHeight) -> Result<()> {
        // Remove spent UTXOs (inputs)
        for input in &tx.inputs {
            self.remove_utxo(&input.prev_output)?;
        }
        
        // Add new UTXOs (outputs)
        for (index, output) in tx.outputs.iter().enumerate() {
            let outpoint = OutPoint::new(tx.id(), index as u32);
            let utxo = UTXO::new(
                output.clone(),
                block_height,
                tx.id(),
                index as u32,
                tx.is_coinbase(),
            );
            self.add_utxo(outpoint, utxo)?;
        }
        
        Ok(())
    }
    
    /// Revert transaction from UTXO set
    pub fn revert_transaction(&mut self, tx: &Transaction, block_height: BlockHeight) -> Result<()> {
        // Remove UTXOs created by this transaction (outputs)
        for (index, _) in tx.outputs.iter().enumerate() {
            let outpoint = OutPoint::new(tx.id(), index as u32);
            self.remove_utxo(&outpoint)?;
        }
        
        // This would require having the UTXOs that were spent to restore them
        // In a real implementation, you'd need to store them or look them up
        // For now, we'll return an error indicating this needs more data
        if !tx.inputs.is_empty() {
            return Err(BlockchainError::StateError(
                "Cannot revert transaction without original UTXOs".to_string()
            ));
        }
        
        Ok(())
    }
}

impl Default for UTXOSet {
    fn default() -> Self {
        Self::new()
    }
}



/// World state combining both account and UTXO models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    ///account-based state (etherium style)
    accounts: IndexMap<Address, AccountState>,
    //utxo-based(bitcoin-style)
    utxo_set: UTXOSet,
    ///state root hash for verification
    state_root: Hash256,
    //Block heigt this state represents
    block_height: BlockHeight,
    ///account model type
    model_type: AccountModel,
}


impl WorldState {
    //create new state
    pub fn new(model_type: AccountModel) -> Self{
        Self{
            accounts: IndexMap::new(),
            utxo_set: UTXOSet::new(),
            state_root: Hash256::zero(),
            block_height: 0,
            model_type,
        }
    }


    ///get account state
    pub fn get_account(&self, address: &Address) -> AccountState {
        self.accounts.get(address)
            .unwrap_or(&AccountState::empty())
    }


    ///get mutabble account state
    pub fn get_account_mut(&mut self, address: &Address) -> &mut AccountState{
        self.accounts.entry(*address)
            .or_insert_with(&AccountState::empty)
    }


    //set account state
    pub fn set_account(&mut self, address: Address, state: AccountState){
        if state.is_empty() {
            self.accounts.shift_remove(&address);

        }else {
            self.accounts.insert(address, state);
        }

        self.invalidate_state_root();
    }


    ///get account balance
    pub fn get_balance(&self, address: &Address) -> Amount {
        match self.model_type{
            AccountModel::Account | AccountModel::Hybrid => {
                self.get_account(address).balance
            }
            AccountModel::UTXO => {
                self.utxo_set.get_balance(address)
            }

        }
    }


    ///transfer btwn accounts (account model)
    pub transfer(&mut self, from: &Address, to: &Address, amount: Amount) -> {
        if amount == 0 {
            return Ok(());
        }


        //get sender account
        let sender_balance = self.get_balance(from);
        if sender_balance< amount {
            return Err(BlockchainError::InsufficientBalance{
                required: amount,
                available: sender_balance,
            });
        }

        let sender_account = self.get_account_mut(from);
        sender_account.sub_balance(amount)?;

        //update recipient account
        let recipient_account = self.get_account_mut(to);
        recipient_account.add_balance(amount)?;

        self.invalidate_state_root();
        Ok(())
    }


    ///Apply transaction to world state
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<()> {
        match self.model_type {
            AccountModel::UTXO => {
                self.apply_utxo_balance(tx)
            }

            AccountModel::Account => {
                self.apply_account_transaction(tx)
            }
            AccountModel::Hybrid =>{

                //try to determine transaction type and apply accordingly
                if !tx.inputs.is_empty() || !tx.outputs.is_empty() {
                    self.apply_utxo_balance(tx)
                }else{
                    self.apply_account_transaction(tx)
                }
            }

        }
    }

    fn apply_utxo_balance(&mut self, tx: &Transaction) -> Result<()> {
        self.utxo_set.apply_transaction(tx, self.block_height)?;
        self.invalidate_state_root();
        Ok(())
    }

    //Apply account-based transaction
    fn apply_account_transaction(&mut self, tx: &Transaction) -> Result<()> {
        //skip coinbase transactions for account model


        ///Coinbase has only the receiver and the amount a fields
        /// as it is gotten after a block is mined and the reward goes to the miner.
        if tx.is_coinbase() {
            if let (Some(to), Some(amount)) = &tx.to, (tx.amount) {
                let account = self.get_account_mut(to);
                account.add_balance(amount)?;
            }

            self.invalidate_state_root();
            return Ok(());
        }

        let from = tx.from.ok_or_else(||
            BlockchainError::InvalidTransaction("Missing sender address".to_string())
            )?;

        let to = tx.to.ok_or_else(||
            BlockchainError::InvalidTransaction("Missing recipient address".to_string())
            )?;

        let amount = tx.amount.unwrap_or(0);
        let gas_fee = tx.calculate_gas_fee();
        let total_cost = amount + gas_fee;


        //check sender balance and nonce
        let sender_state = self.get_account(&from);
        if sender_state.balance < total_cost {
            return Err(
                BlockchainError::InsufficientBalance{
                    required: total_cost,
                    available: sender_state.balance,
                }
                );
        }

        if let Some(tx_nonce) = tx.nonce {
            if sender_state.nonce != tx.nonce {
                return Err(InvalidTransaction(
                    format!("Invalid nonce: expected {}, got {}", sender_state.nonce, tx.nonce)
                    ));
            }
        }


        //apply transaction
        if amount > 0 {
            self.transfer(&from, &to, amount)?;
        }


        //Deduct gas fee
        if gas_fee > 0 {
            let sender_account = self.get_account_mut(&from);
            sender_account.sub_balance(gas_fee)?;
        }


        //increment sender nonce
        let sender_account = self.get_account_mut(&from);
        sender_account.increment_nonce();


        self.invalidate_state_root();
        Ok(())

    }

    //getutxo set
    pub fn utxo_set(&self) -> &UTXOSet {
        &self.utxo_set
    }


    //get mutable utxo set
    pub fn utxo_set_mut(&mut self) -> &mut UTXOSet{
        &mut self.utxo_set
    }

    //get all accounts
    pub fn accounts(&self) -> &IndexMap<Address, AccountState> {
        &self.accounts
    }


    ///Calculate state root hash
    pub fn calculate_state_root_hash(&self) -> Hash256 {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();

        //hash account states
        for (address, account) in &self.accounts {
            let account_data = bincode::serialize(&(address, account)).unwrap_or_default();
            hasher.update(&account_data);
        }

        //hash utxo-set
        let mut utxo_hashes: Vec<_> = self.utxo_set.iter().collect();
        utxo_hashes.sort_by_key(|(outpoint, _)| *outpoint);

        for (outpoint, utxo) in utxo_hashes{
            let utxo_data = bincode::serialize(&(outpoint, utxo)).unwrap_or_default();
            hasher.update(&utxo_data);
        }

        Hash256::from_bytes(hasher.finalize().into())
    }


    ///update and get state root
    pub fn state_root(&mut self) -> Hash256 {
        if self.state_root.is_zero() {
            self.state_root = self.calculate_state_root_hash();

        }
        self.state_root

    }


    ///get all current state root without recalculation
    pub fn current_state_root(&self) -> Hash256 {
        self.state_root
    }


    ///invalidate cached state root
    fn invalidate_state_root(&mut self) {
        self.state_root = Hash256::zero();
    }

    //set block height
    pub fn set_block_height(&mut self, height: BlockHeight) {
        self.block_height = block_height;
    }

    ///get block height
    pub fn get_block_height(&self) -> BlockHeight {
        self.block_height
    }


    /// Create snapshot of current state(Claudie direct)
    pub fn snapshot(&self) -> WorldStateSnapshot {
        WorldStateSnapshot {
            accounts: self.accounts.clone(),
            utxo_set: self.utxo_set.clone(),
            state_root: self.state_root,
            block_height: self.block_height,
        }
    }


    //restore from snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: WorldStateSnapshot) {
        self.accounts = snapshot.accounts;
        self.utxo_set = snapshot.utxo_set;
        self.state_root = snapshot.state_root;
        self.block_height = snapshot.block_height;
    }


    ///get tota; supply(all balances + utxo)
    pub fn total_supply(&self) -> Amount{
        let account_supply: Amount = self.accounts.values()
            .iter()
            .sum();


        let utxo_supply = self.utxo_set.total_value();

        account_supply + utxo_supply
    }


    pub fn validate(&self) -> Result<()> {
        //validate utxo set internal consistency
        let calculated_total: Amount = selt.utxo_set.utxos.values()
            .map(|utxo| utxo.output.amount)
            .sum();

        if calculated_total != self.utxo_set.total_value() {
            return Err(BlockchainError::StateError(
                "UTXO set total value mismatch".to_string()
                ));
        }


        //Validate address index consistency
        for (address, output) in &self.utxo_set.address_index {
            for outpoint in outpoints {
                if let Some(utxo) = self.utxo_set.get(outpoint) {
                    if utxo.output.address != *address {
                        return Err(
                            BlockchainError::StateError(
                                format!("Address index mismatch for {}", address))
                            );
                    }
                }else {
                    return Err(BlockchainError::StateError(
                        format!("Orphaned outpoint in address index: {}", outpoint)
                        ));
                }
            }
            
        }

        Ok(())
    }
}

/// Snapshot of world state for rollback purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStateSnapshot {
    accounts: IndexMap<Address, AccountState>,
    utxo_set: UTXOSet,
    state_root: Hash256,
    block_height: BlockHeight,
}


#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_crypto::{signature::generate_keypair, address::public_key_to_address, AddressType};
    use crate::transaction::TransactionOutput;

    #[test]
    fn test_account_state() {
        let mut account = AccountState::new(1000);
        
        assert_eq!(account.balance, 1000);
        assert_eq!(account.nonce, 0);
        assert!(!account.is_empty());
        
        account.add_balance(500).unwrap();
        assert_eq!(account.balance, 1500);
        
        account.sub_balance(200).unwrap();
        assert_eq!(account.balance, 1300);
        
        account.increment_nonce();
        assert_eq!(account.nonce, 1);
    }

    #[test]
    fn test_utxo_set() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        let mut utxo_set = UTXOSet::new();
        
        // Create a UTXO
        let tx_id = TxId::new(sha256(b"test tx"));
        let outpoint = OutPoint::new(tx_id, 0);
        let output = TransactionOutput::new(1000, address);
        let utxo = UTXO::new(output, 1, tx_id, 0, false);
        
        utxo_set.add_utxo(outpoint, utxo).unwrap();
        
        assert_eq!(utxo_set.len(), 1);
        assert_eq!(utxo_set.get_balance(&address), 1000);
        assert_eq!(utxo_set.total_value(), 1000);
        assert!(utxo_set.contains(&outpoint));
        
        // Remove UTXO
        let removed_utxo = utxo_set.remove_utxo(&outpoint).unwrap();
        assert_eq!(removed_utxo.output.amount, 1000);
        assert_eq!(utxo_set.len(), 0);
        assert_eq!(utxo_set.get_balance(&address), 0);
    }

    #[test]
    fn test_world_state_account_model() {
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        let mut world_state = WorldState::new(AccountModel::Account);
        
        // Set initial balances
        world_state.set_account(addr1, AccountState::new(1000));
        world_state.set_account(addr2, AccountState::new(500));
        
        assert_eq!(world_state.get_balance(&addr1), 1000);
        assert_eq!(world_state.get_balance(&addr2), 500);
        assert_eq!(world_state.total_supply(), 1500);
        
        // Transfer funds
        world_state.transfer(&addr1, &addr2, 200).unwrap();
        
        assert_eq!(world_state.get_balance(&addr1), 800);
        assert_eq!(world_state.get_balance(&addr2), 700);
    }

    #[test]
    fn test_world_state_utxo_model() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        let mut world_state = WorldState::new(AccountModel::UTXO);
        
        // Create a coinbase transaction
        let coinbase_tx = Transaction::new_coinbase(address, 5000000000, 1);
        
        world_state.apply_transaction(&coinbase_tx).unwrap();
        
        assert_eq!(world_state.get_balance(&address), 5000000000);
        assert_eq!(world_state.utxo_set().len(), 1);
    }

    #[test]
    fn test_world_state_snapshot() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        let mut world_state = WorldState::new(AccountModel::Account);
        world_state.set_account(address, AccountState::new(1000));
        
        // Create snapshot
        let snapshot = world_state.snapshot();
        
        // Modify state
        world_state.set_account(address, AccountState::new(2000));
        assert_eq!(world_state.get_balance(&address), 2000);
        
        // Restore from snapshot
        world_state.restore_from_snapshot(snapshot);
        assert_eq!(world_state.get_balance(&address), 1000);
    }

    #[test]
    fn test_insufficient_balance() {
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let addr1 = public_key_to_address(keypair1.public_key(), AddressType::Base58);
        let addr2 = public_key_to_address(keypair2.public_key(), AddressType::Base58);
        
        let mut world_state = WorldState::new(AccountModel::Account);
        world_state.set_account(addr1, AccountState::new(100));
        
        // Try to transfer more than available
        let result = world_state.transfer(&addr1, &addr2, 200);
        assert!(matches!(result, Err(BlockchainError::InsufficientBalance { .. })));
    }
}




