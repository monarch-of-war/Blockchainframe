use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;
// use ed25519_dalek::{PublicKey};




pub type Pubkey = [u8;32];

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Clone)]
pub struct Mint {
	pub decimals: u8,
	pub supply: u128,
	pub mint_authority: Option<Pubkey>,
	pub freeze_authority: Option<Pubkey>,
}

impl Mint{
	pub fn new(decimals: u8, mint_authority: Option<Pubkey>) ->Self{
		Self{decimals, supply: 0, mint_authority, freeze_authority:None}
	}
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Clone)]
pub struct TokenAccount{
	pub owner: Pubkey,
	pub amount: u128,
	pub mint: Pubkey,
}

impl TokenAccount{
	pub fn new(owner: Pubkey, mint: Pubkey) ->Self{
		Self{owner, amount: 0, mint}
	}
}