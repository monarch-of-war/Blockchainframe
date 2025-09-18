use borsh::{BorshDeserialize, BorshSerialize};
use crate::state::Pubkey;


#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Clone)]
pub enum BankInstruction{
	// initialize a mint: decimals, mint_authority
	InitMint{
		decimals: u8,
		mint_authority: Option<Pubkey>,
	},

	// initialize token account: owner pubkey
	InitAccount{owner: Pubkey},


	// transfer smount from source_account to dest_account
	Transfer{amount: u128},

	// Mint tokens to a token account (only mint_authority)
	MintTo{amount: u128},

	// Burn tokens from token account (owner or delegate)

	Burn{amount: u128},
}