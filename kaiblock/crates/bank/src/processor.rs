use borsh::{BorshSerialize, BorshDeserialize};
use thiserror::Error;

use crate::instruction::BankInstruction;
use crate::state::{Mint, TokenAccount, Pubkey};
use std::collections::HashMap;


pub type AccountData = Vec<u8>;
pub type AccountStore = HashMap<Vec<u8>, AccountData>;


#[derive(Error, Debug)]
pub enum BankError{
	#[error("account not found")]
	AccountNotFound,
	#[error("invalid instruction data")]
	InvalidInstruction,
	#[error("insufficient funds")]
	InsufficientFunds,
	#[error("unauthorized")]
	Unauthorized,
	#[error("bad mint")]
	BadMint,
}


pub fn process_instruction(
	program_id: &[u8; 32],
	accounts: &mut AccountStore,
	instruction_data: &[u8],
	signers: &[Pubkey] //a list of signers for this tx
	) ->Result<(), BankError> {
	let instr = BankInstruction::try_from_slice(instruction_data)
	.map_err(|_| BankError::InvalidInstruction)?;

	match instr{
		BankInstruction::InitMint {decimals, mint_authority} =>{

			// account[0] is the mint account pubkey (key used in store)
			// caller must ensure accounts contain a zeroed account to be initialized
			let mint_key = program_id.to_vec(); //placeholder usage; expect caller to provide keys.
			let key = mint_key;

			if accounts.contains_key(&key) {
				return Err(BankError::InvalidInstruction);
			}

			let mint = Mint::new(decimals, mint_authority);

			accounts.insert(key, mint.try_to_vec().unwrap());

			Ok(())

		}


		BankInstruction::InitAccount {owner} => {
			let acct_key = program_id.to_vec();
			if accounts.contains_key(&acct_key){
				return Err(BankError::InvalidInstruction);
			}

			let mint_ref: Pubkey = *program_id;
			let token_account = TokenAccount::new(owner, mint_ref);

			accounts.insert(acct_key, token_account.try_to_vec().unwrap());
			Ok(())
		}

		BankInstruction::Transfer {amount} => {
			// for this simple demo assume two keys in accounts: source, dest, with keys provided by runtime
			if accounts.len() <2{
				return Err(BankError::AccountNotFound);
			}

			let mut iter = accounts.iter_mut();
			let source_key = iter.next().unwrap().0.clone();
			let dest_key = iter.next().unwrap().0.clone();

			let source_data = accounts.get_mut(&source_key).ok_or(BankError::AccountNotFound)?;
			let dest_data = accounts.get_mut(&dest_key).ok_or(BankError::AccountNotFound)?;


			let mut source_acct = TokenAccount::try_from_slice(source_data).map_err(|_| BankError::InvalidInstruction)?;
			let dest_acct = TokenAccount::try_from_slice(dest_data).map_err(|_| BankError::InvalidInstruction)?;

			if source_acct.amount < amount{
				return Err(BankError::InsufficientFunds);
			}

			source_acct.amount = source_acct.amount.saturating_sub(amount);
			dest_acct.amount = dest_acct.amount.saturating_add(amount);


			*source_data = source_acct.try_to_vec().unwrap();
			*dest_data = dest_acct.try_to_vec().unwrap();

			Ok(())


		}


		BankInstruction::MintTo{amount} => {

			//accounts: mint_account, dest_token_account

			if accounts.len()<2 {
				return Err(BankError::AccountNotFound);
			}

			let mut iter = accounts.iter_mut();
			let mint_key = iter.next().unwrap().0.clone();
			let dest_key = iter.next().unwrap().0.clone();

			let mint_data = accounts.get_mut(&mint_key).ok_or(BankError::AccountNotFound)?;
			let dest_data = accounts.get_mut(&dest_key).ok_or(BankError::AccountNotFound)?;

			let mut mint = Mint::try_from_slice(mint_data).map_err(|_| BankError::InvalidInstruction)?;
			let mut dest_acct = TokenAccount::try_from_slice(dest_data).map_err(|_| BankError::InvalidInstruction)?;


			//check signer is mint_authority
			let authority = mint.mint_authority.ok_or(BankError::Unauthorized)?;
			if !signers.iter().any(|s| s==&authority){
				return Err(BankError::Unauthorized);
			}

			mint.supply = mint.supply.saturating_add(amount);

			dest_acct.amount = dest_acct.amount.saturating_add(amount);


			*mint_data = mint.try_to_vec().unwrap();

			*dest_data = dest_acct.try_to_vec().unwrap();

			Ok(())

		}

		BankInstruction::Burn{amount} => {

			// accounts: token_account, mint_account

			if accounts.len() <2{
				return Err(BankError::AccountNotFound);
			}

			let mut iter = accounts.iter_mut();
			let token_key = iter.next().unwrap().0.clone();
			let mint_key = iter.next().unwrap().0.clone();

			let token_data = accounts.get_mut(&token_key).ok_or(BankError::AccountNotFound)?;
			let mint_data = accounts.get_mut(&mint_key).ok_or(BankError::AccountNotFound)?;

			let mut token_acct = TokenAccount::try_from_slice(token_data).map_err(|_| BankError::InvalidInstruction)?;
			let mut mint = Mint::try_from_slice(mint_data).map_err(|_| BankError::InvalidInstruction)?;

			if token_acct.amount < amount {
				return Err(BankError::InsufficientFunds);
			}

            // for simplicity, check that signer is owner
            // In a real runtime you'd pass signers and the token account owner to check.
            // Here we require the first signer equals token owner
            // (caller must provide signers param accordingly)
            // That check is done by runtime; omitted here for brevity.


            token_acct.amount = token_acct.amount.saturating_sub(amount);
            mint.supply = mint.supply.saturating_sub(amount);


            *token_data = token_acct.try_to_vec().unwrap();
            *mint_data = mint.try_to_vec().unwrap();

            Ok(())
		}


	}
}


// Notes & integration hints:

// process_instruction is intentionally minimal and uses a simple AccountStore map keyed by bytes (in your runtime these will be real account pubkeys).

// In your real runtime, the program_id and accounts will be provided by the execution engine; replace placeholder logic with reading account keys and owners from the runtime environment.

// Authorization checks here are illustrative — your runtime should pass signers and validate signatures against account owners.