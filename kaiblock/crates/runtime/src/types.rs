use borsh::{BorshDeserialize, BorshSerialize};

/// 32-byte pubkey alias to keep this crate independent of your crypto type.
/// Replace with your real PublicKey type when wiring into signature code.


pub type Pubkey = [u8; 32];


/// An instruction inside a transaction.
#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Instruction {
	//program id to invoke.
	pub program_id: Pubkey,
	//indeces into the transaction's accounts vector. These refer to the
	// accounts that will be passed to the program in this order.
	pub accounts: Vec<u8>,
	// Opaque instruction data(program specific, typically borsh-encoded)
	pub data: Vec<u8>,
}


// Account information passed to prorams during execution.
// In real runtime, data will be a slice of referencing persisted account storage.

#[derive(Debug, Clone)]
pub struct AccountInfo {
	pub pubkey: Pubkey,
	pub owner: Pubkey,
	pub is_signed: bool,
	pub is_writable: bool,
	pub data: Vec<u8>,
}


/// Canonical transaction structure used by the runtime.
/// `signers` is a list of pubkeys that have signed the transaction – runtime must verify signatures.
#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Transaction {
	pub fee_payer: Pubkey,
	pub recent_blockhash: [u8, 32],
	pub accounts: Vec<AccountMeta>,
	pub instructions: Vec<Instruction>,
	    /// Signatures are not part of core runtime type here; they are handled by node-level code.
    /// For tests we simulate signature presence via the `signers` arg to runtime.execute_transaction.

}


/// Metadata describing accounts referenced by the transaction.
/// The runtime will prepare `AccountInfo` instances for programs using this metadata.
#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct AccountMeta{
	pub pubkey: Pubkey,
	pub owner: Pubkey,
	pub is_signed: bool,
	pub is_writable: bool,
}