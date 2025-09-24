use serde::{Deserialize, Serialize};
use blockchain_crypto::Hash256;
use chrono::{DateTime, Utc};
use std::fmt;


///block height type
pub type BlockchainHeight = u64;

pub type BlockHeight = u64;


///transaction amount type
pub type Amount = u64;


///transaction fee type
pub type Gas = u64;


///Gas pricelimit for smart contracts
pub GasPrice = u64;

///Nonce for preventing replay attacks
pub type Nonce = u64;


///Difficulty target for mining
pub type Difficulty = u64;


///Chain ID for network identification
pub type ChainId = u32;


///Transaction Id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]

pub struct TxId(Hash256);


impl TxId{

	pub fn new(hash: Hash256){
		Self(hash)
	}


	pub fn hash(&self) -> Hash256{
		self.0
	}


	pub fn from_hex(hex: &str) -> blockchain_crypto::Result<Self> {
		Ok(Self(Hash256::from_hex(hex)?))
	}


	pub fn to_hex(&self) -> String{
		self.0.to_hex()
	}


}


impl fmt::Display for TxId {
	fn fmt(&self, f: &mut fmt::Result<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}


impl From<Hash256> for TxId {
	fn from(hash: Hash256) -> Self {
		Self(hash)
	}
}

impl AsRef<Hash256> for TxId {
	fn as_ref(&self) -> &Hash256 {
		&self.0
	}
}

///Block ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockID(Hash256);


impl BlockID{
	pub fn new(hash: Hash256) -> Self {
		Self(hash)
	}


	pub fn hash(&self) -> Hash256 {
		self.0
	}

	pub fn from_hex(hex: &str) -> blockchain_crypto::Result<Self>{
		Ok(Self(Hash256::from_hex(hex)?))
	}


	pub fn to_hex(&self) -> String {
		self.0.to_hex()
	}


	pub fn genesis() -> Self {
		Self(Hash256::zero())
	}
}


impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Hash256> for BlockId {
    fn from(hash: Hash256) -> Self {
        Self(hash)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp{
	pub fn now() ->Self{
		Self(Utc::now())
	}

	pub fn from_unix_timestamp(timestamp: i64) -> Self{
		Self(DateTime::from_timestamp(timestamp, 0).unwrap_or_default())
	}


	pub fn to_unix_timestamp(&self) -> i64{
		self.0.timestamp()
	}


	pub fn inner(&self) -> DateTime<Utc>{
		self.0
	}


}



impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}


///account model type for state management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountModel{
	///UTXO model like bitcoin
	UTXO,

	///account-based model like etherium
	Account,

	///Hybrid model supporting both
	Hybrid,
}


///Transaction type clasification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType{
	///regular transfer transaction
	Transfer,
	///coinbase transaction as mining reward
	Coinbase,
	///smart contract deployment
	ContractDeployment,
	///smart contract execution
	ContractCall,
	///multi signature transactions
	Multisig,
}


///network types for different environments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub NetworkType {
	///main production network
	Mainnet,
	///test network
	Testnet
	///development network
	Devnet,
	///local development
	Local,
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockStatus {
	///block pending verification
	Pending,
	///block is valid and confirmed
	Confirmed,
	///block is invalid
	Invalid,
	///Block is orphaned(not in mainchain)
	Orphaned,
}



///transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus{
	Pending,
	Confirmed,
	Failed,
	//dropped from mempool
	Dropped,
}


///utxo reference for inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutPoint {
	///transaction hash containinf utxo
	pub tx_id: TxId,
	///output cindex within the transaction
	pub output_index: u32;
}


impl OutPoint {
    pub fn new(tx_id: TxId, output_index: u32) -> Self {
        Self { tx_id, output_index }
    }
}

impl fmt::Display for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.tx_id, self.output_index)
    }
}



/// Script for transaction validation (simplified)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Script {
    /// Pay to public key hash (P2PKH)
    PayToPubkeyHash(Hash256),
    /// Pay to script hash (P2SH)  
    PayToScriptHash(Hash256),
    /// Pay to public key
    PayToPubkey(blockchain_crypto::PublicKey),
    /// Multi-signature script
    MultiSig {
        threshold: u8,
        public_keys: Vec<blockchain_crypto::PublicKey>,
    },
    /// Custom script bytecode
    Custom(Vec<u8>),
}


impl Script {
    /// Create a simple P2PKH script
    pub fn pay_to_pubkey_hash(address_hash: Hash256) -> Self {
        Script::PayToPubkeyHash(address_hash)
    }
    
    /// Create a P2PK script
    pub fn pay_to_pubkey(public_key: blockchain_crypto::PublicKey) -> Self {
        Script::PayToPubkey(public_key)
    }
    
    /// Create a multi-sig script
    pub fn multi_sig(threshold: u8, public_keys: Vec<blockchain_crypto::PublicKey>) -> Self {
        Script::MultiSig { threshold, public_keys }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_crypto::hash::sha256;

    #[test]
    fn test_tx_id() {
        let hash = sha256(b"test transaction");
        let tx_id = TxId::new(hash);
        
        assert_eq!(tx_id.hash(), hash);
        assert_eq!(TxId::from(hash), tx_id);
    }

    #[test]
    fn test_block_id() {
        let hash = sha256(b"test block");
        let block_id = BlockId::new(hash);
        
        assert_eq!(block_id.hash(), hash);
        assert_ne!(block_id, BlockId::genesis());
    }

    #[test]
    fn test_timestamp() {
        let now = Timestamp::now();
        let unix_timestamp = now.to_unix_timestamp();
        let restored = Timestamp::from_unix_timestamp(unix_timestamp);
        
        // Should be within 1 second due to potential precision loss
        assert!((now.to_unix_timestamp() - restored.to_unix_timestamp()).abs() <= 1);
    }

    #[test]
    fn test_outpoint() {
        let tx_id = TxId::new(sha256(b"test tx"));
        let outpoint = OutPoint::new(tx_id, 0);
        
        assert_eq!(outpoint.tx_id, tx_id);
        assert_eq!(outpoint.output_index, 0);
    }
}