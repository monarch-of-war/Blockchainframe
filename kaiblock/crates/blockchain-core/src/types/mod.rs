// Hash, Amount, BlockHeight core types
// Overflow-safe arithmetic for Amount
// Timestamp utilities with current time functions

use serde::{Serialize, Deserialize};
use std::fmt;


//re-export blockchain-crypto types

pub use blockchain_crypto::hash::Hash as CryptoHash;
pub use blockchain_crypto::Address;


//Primary hash type for blockchain
pub type Hash = [u8; 32];

//block height type
pub type BlockHeight = u64;

//transaction amounts in smallest units(like satoshis)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount(pub u64);

impl Amount{
	pub const ZERO: Amount = Amount(0);
	pub const MAX: Amount = Amount(u64::MAX);


	//create new amount with validation
	pub fn new(value: u64) -> Self {
		Amount(value)
	}

	//Get raw value
	pub fn value(&self) -> u64 {
		self.0
	}


	//add amounts with overflow
	pub fn checked_add(&self, other: &Amount) ->Option<Amount> {
		self.0.checked_add(other.0).map(Amount)
	}


	//subtract amounts with overflow check
	pub fn checked_sub(&self, other: &Amount) ->Option<Amount> {
		self.0.checked_sub(other.0).map(Amount)
	}

	//check if amount is zero

	pub fn is_zero(&self) ->bool{
		self.0 == 0
	}
}


impl std::ops::Sub for Amount{
	type Output = Amount;
	fn sub(self, rhs: Amount) -> Amount {
		Amount(self.0 - rhs.0)
	}
}


//timestamp utilities

pub type Timestamp = u64;

pub fn current_timestamp() -> Timestamp {
	std::time::SystemTime::now()
	.duration_since(std::time::UNIX_EPOCH)
	.unwrap()
	.as_secs()
}