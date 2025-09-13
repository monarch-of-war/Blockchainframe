// blockchain-core/src/consensus/pos.rs
//! Reference Proof-of-Stake consensus engine.
//!
//! - Implements `ConsensusEngine<Block<Tx>>` for a generic Block type.
//! - Minimal but complete: proposer selection, header signing, verify+commit.
//! - Uses an in-memory StakingState (wrap in DB-backed adapter for production).
//!
//! IMPORTANT: This is a reference implementation. Real PoS production systems
//! need slashing, long-range protection, epoch transitions, vote aggregation,
//! time/slot synchronization, and careful economic parameters.

use std::collections::BTreeMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::consensus::{ConsensusEngine, BlockStore, ProductionParams, ConsensusError};
use crate::ledger::block::Block;
use crate::ledger::transaction::TransactionTrait;
use crate::ledger::merkle::merkle_root;

use blockchain_crypto::hash::sha256;
use blockchain_crypto::signature::{PublicKey, Signature, Keypair, sign_message, verify_signature};

use num_bigint::BigUint;
use num_traits::ToPrimitive;

/// Validator entry representing a bonded validator in the set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: PublicKey,
    pub stake: u128,
    pub active: bool,
}

impl Validator {
    pub fn key_bytes(&self) -> Vec<u8> {
        self.pubkey.as_bytes().to_vec()
    }
}

/// In-memory staking state container. Production systems should back this by persistent storage.
#[derive(Debug, Default)]
pub struct StakingState {
    /// Map from public-key-bytes -> Validator
    pub validators: BTreeMap<Vec<u8>, Validator>,
    /// Cached total active stake
    pub total_stake: u128,
}

impl StakingState {
    pub fn add_validator(&mut self, v: Validator) {
        let key = v.key_bytes();
        if let Some(existing) = self.validators.get(&key) {
            // update stake and active flag; naive merge
            let mut new = existing.clone();
            new.stake = new.stake.saturating_add(v.stake);
            new.active = v.active;
            self.total_stake = self.total_stake.saturating_add(v.stake);
            self.validators.insert(key, new);
        } else {
            self.total_stake = self.total_stake.saturating_add(v.stake);
            self.validators.insert(key, v);
        }
    }

    pub fn remove_validator(&mut self, pubkey: &PublicKey) {
        let key = pubkey.as_bytes().to_vec();
        if let Some(v) = self.validators.remove(&key) {
            self.total_stake = self.total_stake.saturating_sub(v.stake);
        }
    }

    pub fn get_active_validators(&self) -> Vec<Validator> {
        self.validators.values().cloned().filter(|v| v.active).collect()
    }

    pub fn get_validator(&self, pubkey: &PublicKey) -> Option<Validator> {
        self.validators.get(&pubkey.as_bytes().to_vec()).cloned()
    }
}

/// PoS engine configuration
#[derive(Debug, Clone)]
pub struct PoSConfig {
    /// Blocks per epoch (used for epoch transitions)
    pub epoch_length: u64,
    /// Required attestation stake percentage (0..100)
    pub required_quorum_pct: u8,
}

/// A compact proposer descriptor returned from proposer selection.
#[derive(Debug, Clone)]
pub struct Proposer {
    pub validator: Validator,
}

/// PoS engine implementation.
///
/// Generic over:
/// - Tx: transaction type implementing TransactionTrait
/// - Store: a BlockStore<Block<Tx>> adapter
pub struct PoSEngine<Tx, Store>
where
    Tx: TransactionTrait + Send + Sync + 'static,
    Store: BlockStore<Block<Tx>> + Send + Sync + 'static,
{
    pub config: PoSConfig,
    pub store: Arc<Store>,
    pub staking: Arc<RwLock<StakingState>>,
    pub node_keypair: Option<Keypair>,
    /// Transaction validation pipeline (e.g. account model, UTXO model, etc.)
    pub validation_pipeline: Arc<dyn ValidationPipeline<Tx>>,
}


impl<Tx, Store> PoSEngine<Tx, Store>
where
    Tx: TransactionTrait + Send + Sync + 'static,
    Store: BlockStore<Block<Tx>> + Send + Sync + 'static,
{
    /// Create a new PoS engine instance
    pub fn new(
        config: PoSConfig,
        store: Arc<Store>,
        staking: Arc<RwLock<StakingState>>,
        node_keypair: Option<Keypair>,
        validation_pipeline: Arc<dyn ValidationPipeline<Tx>>,
    ) -> Self {
        Self { 
            config, 
            store, 
            staking, 
            node_keypair,
            validation_pipeline,
            
        }
    }

    /// Deterministic, stake-weighted proposer selection for a slot.
    ///
    /// Simple algorithm:
    /// - Build entropy = sha256(slot || last_block_hash)
    /// - Convert entropy to a BigUint `e`
    /// - Take `cursor = e % total_stake`
    /// - Walk validators in deterministic order, subtracting stake until cursor < stake
    ///
    /// This is auditable and deterministic; replace with VRF for stronger randomness.
    async fn select_proposer(&self, slot: u64, last_hash: &[u8]) -> Option<Proposer> {
        let staking = self.staking.read().await;
        let validators = staking.get_active_validators();
        if validators.is_empty() || staking.total_stake == 0 {
            return None;
        }

        // entropy = sha256(slot || last_hash)
        let mut input = Vec::with_capacity(8 + last_hash.len());
        input.extend_from_slice(&slot.to_le_bytes());
        input.extend_from_slice(last_hash);
        let entropy = sha256(&input);

        let e = BigUint::from_bytes_be(&entropy);
        let total = BigUint::from(staking.total_stake);
        if total.is_zero() {
            return None;
        }
        let mut cursor = (&e) % &total;

        for v in validators {
            let stake_big = BigUint::from(v.stake);
            if cursor < stake_big {
                return Some(Proposer { validator: v });
            } else {
                cursor -= stake_big;
            }
        }

        // Fallback: deterministic first validator
        staking.validators.values().next().cloned().map(|v| Proposer { validator: v })
    }

    /// Node-local helper: sign header bytes if we have a keypair.
    fn sign_header_bytes(&self, header_bytes: &[u8]) -> Option<Signature> {
        let kp = self.node_keypair.as_ref()?;
        Some(sign_message(header_bytes, kp))
    }

    /// Verify a proposer signature / attestation for the header.
    fn verify_proposer_attestation(pubkey: &PublicKey, header_bytes: &[u8], sig: &Signature) -> bool {
        verify_signature(header_bytes, sig, pubkey)
    }

    /// Build consensus_data blob for a signed header (pubkey + signature).
    fn pack_consensus_data(proposer_pub: &PublicKey, sig: &Signature) -> Result<Vec<u8>, ConsensusError> {
        bincode::serialize(&(proposer_pub.clone(), sig.clone()))
            .map_err(|e| ConsensusError::Internal(format!("consensus_data serialize: {}", e)))
    }
}

#[async_trait]
impl<Tx, Store> ConsensusEngine<Block<Tx>> for PoSEngine<Tx, Store>
where
    Tx: TransactionTrait + Send + Sync + 'static,
    Store: BlockStore<Block<Tx>> + Send + Sync + 'static,
{
    /// Produce a block if this node is the selected proposer for the slot.
    /// If not proposer, returns ConsensusError::Unauthorized.
    async fn produce_block(&self, params: ProductionParams) -> Result<Block<Tx>, ConsensusError> {
        // fetch tip (parent) hash
        let prev_hash = self.store.tip_hash().await.unwrap_or_else(|| vec![]);
        // TODO: pull transactions from mempool up to params.max_txs
        let transactions = self.mempool.fetch(params.max_txs).await;
        let merkle = merkle_root(&transactions);

        let header = crate::ledger::block::BlockHeader {
            prev_hash: prev_hash.clone(),
            merkle_root: merkle,
            timestamp: params.timestamp,
            version: 1,
        };

        let slot = params.timestamp; // simple slot mapping; production systems have dedicated slot numbering
        let proposer = self.select_proposer(slot, &prev_hash).await
            .ok_or_else(|| ConsensusError::Internal("no proposer available".into()))?;

        // If the local node isn't a validator or not the selected proposer -> Unauthorized
        let my_pub_opt = self.node_keypair.as_ref().map(|k| k.public.clone());
        if let Some(my_pub) = my_pub_opt {
            if my_pub.as_bytes() != proposer.validator.pubkey.as_bytes() {
                return Err(ConsensusError::Unauthorized("not proposer for slot".into()));
            }

            // We are proposer: sign header and construct block
            let header_bytes = bincode::serialize(&header)
                .map_err(|e| ConsensusError::Internal(format!("header serialize: {}", e)))?;
            let sig = self.sign_header_bytes(&header_bytes)
                .ok_or_else(|| ConsensusError::Internal("missing node keypair".into()))?;

            let consensus_blob = Self::pack_consensus_data(&proposer.validator.pubkey, &sig)?;

            let block = Block {
                header,
                transactions,
                consensus_data: consensus_blob,
            };

            Ok(block)
        } else {
            Err(ConsensusError::Unauthorized("node has no keypair to sign; cannot produce".into()))
        }

        self.mempool.remove(&transactions).await;
        Ok(())
    }

    /// Validate consensus proof + merkle root + commit block to store.
    // async fn validate_and_commit(&self, block: Block<Tx>) -> Result<(), ConsensusError> {
    //     // 1) verify merkle root
    //     let expected = merkle_root(&block.transactions);
    //     if expected != block.header.merkle_root {
    //         return Err(ConsensusError::InvalidBlock("merkle root mismatch".into()));
    //     }

    //     // 2) decode consensus_data => (pubkey, signature)
    //     let (pubkey, sig): (PublicKey, Signature) = bincode::deserialize(&block.consensus_data)
    //         .map_err(|e| ConsensusError::InvalidBlock(format!("consensus_data decode: {}", e)))?;

    //     // 3) check pubkey is active validator
    //     let staking = self.staking.read().await;
    //     let maybe_validator = staking.get_validator(&pubkey)
    //         .ok_or_else(|| ConsensusError::Unauthorized("proposer not in validator set".into()))?;
    //     if !maybe_validator.active {
    //         return Err(ConsensusError::Unauthorized("proposer is not active".into()));
    //     }

    //     // 4) verify signature
    //     let header_bytes = bincode::serialize(&block.header)
    //         .map_err(|e| ConsensusError::Internal(format!("header serialize: {}", e)))?;
    //     if !Self::verify_proposer_attestation(&maybe_validator.pubkey, &header_bytes, &sig) {
    //         return Err(ConsensusError::InvalidBlock("invalid proposer signature".into()));
    //     }

    //     // 5) OPTIONAL: check proposer selection correctness (recompute who should have been proposer)
    //     // Recompute proposer for the block's slot (timestamp) and ensure it matches pubkey.
    //     let recomputed = {
    //         // NOTE: take a cloned read lock for deterministic calculation
    //         drop(staking); // drop read guard to avoid deadlock if select_proposer takes lock
    //         let prev_hash = block.header.prev_hash.clone();
    //         // select_proposer is async and acquires read lock internally
    //         self.select_proposer(block.header.timestamp, &prev_hash).await
    //     };

    //     if let Some(chosen) = recomputed {
    //         if chosen.validator.pubkey.as_bytes() != pubkey.as_bytes() {
    //             return Err(ConsensusError::InvalidBlock("proposer not selected for this slot".into()));
    //         }
    //     } else {
    //         return Err(ConsensusError::InvalidBlock("failed to recompute proposer".into()));
    //     }

    //     // 6) append to store
    //     self.store.append_block(block).await.map_err(|e| e)
    // }

    pub async fn validate_and_commit<Tx>(&self, block: &Block<Tx>) -> ConsensusResult<()>
    where
        Tx: AccountTransaction + 'static,
    {
        // 1. Verify proposer signature & block link (unchanged)
        self.verify_block_header(block)?;

        // 2. Validate each transaction using the pipeline
        for tx in &block.transactions {
            self.validation_pipeline.validate_and_apply(tx).await?;
        }

        // 3. Append the block only if all txs are valid
        self.store.append_block(block.clone()).await?;
        Ok(())
    }






    /// Lightweight header-check used for gossip filtering: verify signature is valid.
    fn verify_header(&self, header_bytes: &[u8], consensus_data: &[u8]) -> Result<(), ConsensusError> {
        // decode consensus_data but do not consult staking DB (keeps this fast)
        let (pubkey, sig): (PublicKey, Signature) = bincode::deserialize(consensus_data)
            .map_err(|e| ConsensusError::InvalidBlock(format!("consensus_data decode: {}", e)))?;

        if !verify_signature(header_bytes, &sig, &pubkey) {
            return Err(ConsensusError::InvalidBlock("invalid proposer signature".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::transaction::MockTransaction;
    use crate::consensus::ProductionParams;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Minimal in-memory BlockStore for tests
    struct InMemoryStore {
        tip: Option<Vec<u8>>,
        blocks: Vec<Vec<u8>>,
    }

    #[async_trait]
    impl BlockStore<Block<MockTransaction>> for InMemoryStore {
        async fn tip_hash(&self) -> Option<Vec<u8>> {
            self.tip.clone()
        }
        async fn get_block(&self, _hash: &[u8]) -> Option<Block<MockTransaction>> {
            None
        }
        async fn append_block(&self, _block: Block<MockTransaction>) -> Result<(), ConsensusError> {
            Ok(())
        }
    }

    // NOTE: this test assumes existence of MockTransaction and a simple signature API in blockchain_crypto.
    #[tokio::test]
    async fn pos_produce_and_validate_roundtrip() {
        // create a node keypair (test helper in crypto lib)
        let kp = Keypair::generate(); // assume this exists in your crypto lib for tests
        let pub_bytes = kp.public.as_bytes().to_vec();

        // build staking state with one validator (the node itself)
        let mut ss = StakingState::default();
        ss.add_validator(Validator { pubkey: kp.public.clone(), stake: 1_000u128, active: true });

        let staking = Arc::new(RwLock::new(ss));
        let store = Arc::new(InMemoryStore { tip: None, blocks: vec![] });

        let pos = PoSEngine::<MockTransaction, InMemoryStore> {
            config: PoSConfig { epoch_length: 100, required_quorum_pct: 66 },
            store: store.clone(),
            staking: staking.clone(),
            node_keypair: Some(kp.clone()),
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let params = ProductionParams { timestamp: now, max_txs: 10, extra: vec![] };

        // produce block (node is validator and proposer)
        let block = pos.produce_block(params).await.expect("should produce block");

        // validate & commit (this will call select_proposer again and append)
        pos.validate_and_commit(block).await.expect("should validate and commit");
    }
}
