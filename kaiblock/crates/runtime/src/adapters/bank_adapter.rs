//! Adapter that allows the `bank::processor::process_instruction` function
//! to run as a native program under this runtime.
//!
//! The bank processor expects an `AccountStore` mapping of account key -> account bytes.
//! The adapter converts `AccountInfo` -> `AccountStore` for the call, and maps results back.


use crate::program::Program;
use crate::types::AccountInfo;
use crate::executoe::RuntimeContext;


use bank::instruction::BankInstruction;
use bank::processor;
use bank::state::Pubkey as BankPubkey;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;



/// Program id you will use to register the bank program.
/// Should match the program_id used for transactions that invoke bank instructions.

pub const BANK_PROGRAM_ID: BankPubkey = [7u8;32];

pub struct BankProgramAdapter{}

impl BankProgramAdapter{
	pub new()-> Self{
		Self{}
	}
}

impl Program for BankProgramAdapter{
	fn process(
		&self,
		accounts: &mut [AccountInfo],
		data: &[u8],
		ctx: &mut RuntimeContext,
		) ->Result<(), crate::program::ProgramError> {

		//convert instruction data to BankInstruction (borsh)

		let inst = BankInstruction::try_from_slice(data)
			.map_err(| e | crate::program::ProgramError::Custom(format!("borsh decode: {:?}", e)))?;

		// We'll key accounts by their pubkey bytes (Vec<u8>).
        // The bank processor expects the accounts required for a particular instruction in a specific order.
        for acct in accounts.iter(){
        	store.insert(acct.pubkey.to_vec(), acct.data.clone());
        }

        // For signers, the bank processor expects a signers slice of Pubkey (32-byte arrays).
        // The runtime currently doesn't provide the signers list into the Program trait; but we can
        // infer signer presence from AccountInfo.is_signer. We build a signer list from the accounts slice.

        let mut signers: Vec<BankPubkey> = Vec::new();

        for acct in accounts.iter() {
        	if acct.is_signer {
        		signers.push(acct.pubkey);
        	}
        }
        // Call into bank processor
        // Note: bank::processor::process_instruction expects program_id (we'll pass BANK_PROGRAM_ID),
        // accounts store, instruction data, and signers.

        let program_id = BANK_PROGRAM_ID;

        processor::process_instruction(
        	&program_id,
        	&mut store,
        	&data,
        	&signers,
        	).map_err(|e| crate::program::ProgramError::Custom(format!("bank error {:?}", e)))?;

		// Write back changes to the accounts slice (for writable accounts only).
		for acct in accounts.iter_mut() {
			if acc.is_writable {
				if let Some(new_data) = store.get(&acct.pubkey.to_vec()){
					acct.data = new_data.clone();
				}
			}
		}

 // consume a small amount of compute for account writes (example)
		let _ = ctx.consume(50).map_err(|_| crate::program::ProgramError::Custom(format!("compute exhausted".into())))?;


		Ok(())
	}
}
