// // blockchain-core/src/genesis.rs
// //! Genesis block builder and initial chain bootstrap helpers.
// //!
// //! Creates a genesis block, writes it to the provided BlockStore, and initializes
// //! the staking/validator set (in-memory or DB-backed adapter).

// use std::sync::Arc;
// use async_trait::async_trait;
// use tokio::sync::RwLock;
// use thiserror::Error;

// use crate::ledger::block::{Block, BlockHeader, merkle_root};
// use crate::ledger::transaction::TransactionTrait;
// use crate::consensus::{BlockStore, ConsensusError};
// use crate::consensus::pos::{Validator, StakingState};

// /// Genesis options for building the initial chain.
// pub struct GenesisConfig<Tx> {
//     /// Initial list of transactions to include in genesis (can be empty).
//     pub initial_txs: Vec<Tx>,
//     /// Initial validator set for PoS (may be empty for permissionless chains).
//     pub initial_validators: Vec<Validator>,
//     /// Unix timestamp (seconds) for genesis block.
//     pub genesis_time: u64,
//     /// Protocol version for genesis header.
//     pub version: u32,
// }

// impl<Tx> Default for GenesisConfig<Tx> {
//     fn default() -> Self {
//         Self {
//             initial_txs: Vec::new(),
//             initial_validators: Vec::new(),
//             genesis_time: 0,
//             version: 1,
//         }
//     }
// }

// /// Build and persist genesis block, and initialize staking state.
// ///
// /// - `store` must implement `BlockStore<Block<Tx>>` and be ready to accept `append_block`.
// /// - `staking_state` is the same `Arc<RwLock<StakingState>>` used by PoS engine; it will be populated.
// /// - Returns the created `Block<Tx>` on success.
// pub async fn build_and_persist_genesis<Tx, Store>(
//     store: Arc<Store>,
//     staking_state: Arc<RwLock<StakingState>>,
//     cfg: GenesisConfig<Tx>,
// ) -> Result<Block<Tx>, ConsensusError>
// where
//     Tx: TransactionTrait + Send + Sync + Clone + 'static,
//     Store: BlockStore<Block<Tx>> + Send + Sync + 'static,
// {
//     // 1) compute merkle root from initial transactions
//     let merkle = merkle_root(&cfg.initial_txs);

//     // 2) build the canonical header (prev_hash empty for genesis)
//     let header = BlockHeader {
//         prev_hash: vec![], // no parent
//         merkle_root: merkle.clone(),
//         timestamp: cfg.genesis_time,
//         version: cfg.version,
//     };

//     // 3) serialize initial validator set into consensus_data for transparency.
//     //    This is optional; consensus engines may instead persist staking state separately.
//     let consensus_data = if cfg.initial_validators.is_empty() {
//         Vec::new()
//     } else {
//         match bincode::serialize(&cfg.initial_validators) {
//             Ok(bytes) => bytes,
//             Err(e) => return Err(ConsensusError::Internal(format!("validator serialize: {}", e))),
//         }
//     };

//     // 4) construct genesis block
//     let genesis_block = Block {
//         header,
//         transactions: cfg.initial_txs.clone(),
//         consensus_data,
//     };

//     // 5) persist block
//     store
//         .append_block(genesis_block.clone())
//         .await
//         .map_err(|e| ConsensusError::Internal(format!("append_block failed: {:?}", e)))?;

//     // 6) populate staking state
//     {
//         let mut staking = staking_state.write().await;
//         staking.validators.clear();
//         staking.total_stake = 0;
//         for v in cfg.initial_validators.into_iter() {
//             staking.add_validator(v);
//         }
//     }


//     use crate::state::State;

//     pub async fn seed_genesis_accounts<Tx, S>(
//         state: &S,
//         initial_validators: &[Validator],
//         initial_balance: u128,
//     ) -> Result<(), ConsensusError>
//     where
//         Tx: TransactionTrait + Send + Sync + Clone + 'static,
//         S: State + Send + Sync,
//     {
//         for v in initial_validators {
//             state.credit(&v.pubkey, initial_balance).await?;
//             state.set_nounce(&v.pubkey, 0).await?;
//         }
//         Ok(())
//     }

//     Ok(genesis_block)
// }



// blockchain-core/src/genesis.rs
//! Genesis block builder and initial chain bootstrap helpers.
//!
//! Creates a genesis block, writes it to the provided BlockStore, initializes
//! staking/validator state, and seeds account balances/nonces for the validation pipeline.

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use thiserror::Error;

use crate::ledger::block::{Block, BlockHeader, merkle_root};
use crate::ledger::transaction::TransactionTrait;
use crate::consensus::{BlockStore, ConsensusError};
use crate::consensus::pos::{Validator, StakingState};
use crate::state::State;

/// Genesis options for building the initial chain.
pub struct GenesisConfig<Tx> {
    /// Initial list of transactions to include in genesis (can be empty).
    pub initial_txs: Vec<Tx>,
    /// Initial validator set for PoS (may be empty for permissionless chains).
    pub initial_validators: Vec<Validator>,
    /// Unix timestamp (seconds) for genesis block.
    pub genesis_time: u64,
    /// Protocol version for genesis header.
    pub version: u32,
    /// Initial balance to credit each validator account
    pub initial_validator_balance: u128,
}

impl<Tx> Default for GenesisConfig<Tx> {
    fn default() -> Self {
        Self {
            initial_txs: Vec::new(),
            initial_validators: Vec::new(),
            genesis_time: 0,
            version: 1,
            initial_validator_balance: 1_000_000, // default
        }
    }
}

/// Build and persist genesis block, initialize staking state, and seed account balances.
pub async fn build_and_persist_genesis<Tx, Store, S>(
    store: Arc<Store>,
    staking_state: Arc<RwLock<StakingState>>,
    state: Arc<S>,
    cfg: GenesisConfig<Tx>,
) -> Result<Block<Tx>, ConsensusError>
where
    Tx: TransactionTrait + Send + Sync + Clone + 'static,
    Store: BlockStore<Block<Tx>> + Send + Sync + 'static,
    S: State + Send + Sync + 'static,
{
    // 1) compute merkle root from initial transactions
    let merkle = merkle_root(&cfg.initial_txs);

    // 2) build the canonical header (prev_hash empty for genesis)
    let header = BlockHeader {
        prev_hash: vec![], // no parent
        merkle_root: merkle.clone(),
        timestamp: cfg.genesis_time,
        version: cfg.version,
    };

    // 3) serialize initial validator set into consensus_data for transparency
    let consensus_data = if cfg.initial_validators.is_empty() {
        Vec::new()
    } else {
        match bincode::serialize(&cfg.initial_validators) {
            Ok(bytes) => bytes,
            Err(e) => return Err(ConsensusError::Internal(format!("validator serialize: {}", e))),
        }
    };

    // 4) construct genesis block
    let genesis_block = Block {
        header,
        transactions: cfg.initial_txs.clone(),
        consensus_data,
    };

    // 5) persist block
    store
        .append_block(genesis_block.clone())
        .await
        .map_err(|e| ConsensusError::Internal(format!("append_block failed: {:?}", e)))?;

    // 6) populate staking state
    {
        let mut staking = staking_state.write().await;
        staking.validators.clear();
        staking.total_stake = 0;
        for v in cfg.initial_validators.iter() {
            staking.add_validator(v.clone());
        }
    }

    // 7) seed validator accounts in state for validation pipeline
    for v in cfg.initial_validators.iter() {
        state
            .credit(&v.pubkey, cfg.initial_validator_balance)
            .await
            .map_err(|e| ConsensusError::Internal(format!("state credit failed: {:?}", e)))?;
        state
            .set_nonce(&v.pubkey, 0)
            .await
            .map_err(|e| ConsensusError::Internal(format!("state nonce set failed: {:?}", e)))?;
    }

    Ok(genesis_block)
}
