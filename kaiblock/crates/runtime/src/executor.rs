0use crate::types::*;
use crate::program::{Program, ProgramError};
use std::collections::HashMap;
use thiserror::Error;
use std::sync::Arc;
use log::info;


/// Configuration for the runtime (budgets, etc.)
#[derive(Clone)]
pub struct RuntimeConfig {
	//initial compute units provided by the transaction
	pub max_compute_units: u64,
	//cost per instruction byte(simple model)
	pub byte_cost: u64,
	// cost per instruction (flat)
	pub instr_cost: u64,
}


impl Defaulf for RuntimeConfig{
	fn default() ->Self{
		Self{
			max_compute_units: 1_000_000,
			byte_cost: 10,
			instr_cost: 500,
		}
	}
}

//Runtime context handed to programs for limited host functionality
pub struct RuntimeContext{
	//Remaining compute units available for the transaction
	pub remaining_compute: u64,
	//Access to logs via log::info; additional host functions can be added.
	pub clock: u64; //slot/timestamp;runtime sets this.
}


impl RuntimeContext{
	pub fn consume(&mut self, amount: u64) ->Result<(), RuntimeError> {
		if self.remaining_compute< amount{
			return Err(RuntimeError::ComputeBudgetExceeded);
		}

		self.remaining_compute -= amount;
		Ok(())
	}

	pub fn log(&sekf, msg: &str){
        // delegated to log crate; programs should use ctx.log for deterministic logging
		info!("{}", msg)
	}
}


#[derive(Error, Debug)]
pub enum RuntimeError{
	#[error("program not found")]
	ProgramNotFound,
	#[error("account index out of bounds")]
	AccountIndexOOB,
	#[error("compute budget exceede")]
	ComputeBudgetExceeded,
	#[error("program error: {0}")]
	ProgramError(String),
    #[error("transaction signature verification failed")]
    SignatureVerificationFailed,
    #[error("invalid instruction data: {0}")]
    InvalidInstructionData(String),
}


/// The runtime holds a registry of programs (native adapters or WASM shims).
pub struct Runtime {
	programs: HashMap<Pubkey, Arc<dyn Program>>,
	config: RuntimeConfig,
	// for tests/dev only: simulated clock(slot/timestamp)

	pub clock: u64;
}

impl Runtime {
	pub fn new(config: RuntimeConfig)->Self{
		Self{
			programs: HashMap::new(),
			config,
			clock: 0,
		}
	}

	//register a native program
	pub fn register_program<P: Program+ 'static >(&mut self, program_id: Pubkey, program: P){
		self.programs.insert(program_id, Arc::new(program));
	}


// execute a transaction. `signers` are pubkeys included as signers for this tx (runtime is expected to verify signatures)
// before calling this; tests will use this param to simulate signature presence.

	pub fn execute_transaction(
		&mut self,
		tx: &Transaction,
		signers: &[Pubkey],
		) ->Result<(), RuntimeError>{
	        // Here we allow caller to simulate that signers have been validated.
	        // In production: verify signatures, check fee payer balance, nonce/recent-blockhash, etc.
	        // For now, sample check: require fee_payer to be present in signers.
	        if !signers.iter().any(|s| s==&tx.fee_payer) {
	        	return Err(RuntimeError::SignatureVerificationFailed);
	        }

	        // Build account infos map (pubkey -> AccountInfo). We'll clone metadata into AccountInfo
	        // The transaction's AccountMeta list is the authoritative ordering of accounts for programs.

	        let mut account_map: HashMap<Pubkey, AccountInfo> = HashMap::new();

	        for meta in &tx.accounts {
	        	//initialize empty data for account unless it already has data in map(test harness may pre-populate)
	        	let ai = AccountInfo{
	        		pubkey: meta.pubkey,
	        		owner: meta.owner,
	        		is_signer: meta.is_signer,
	        		is_writable: meta.is_writable,
	        		data: vec![], //this would be the accounts persisted bytes in real node.
	        	};

	        	account_map.insert(meta.pubkey, ai)
	        }


	        //prepare runtime context
	        let mut ctx = RuntimeContext{
	        	remaining_compute: self.config.max_compute_units,
	        	clock: self.clock,
	        };


	        for instr in &tx.instruction {
	        	//compute cost estimation: instr_cost + byte_cost * data_len
	        	let data_cost = (instr.data.len() as u64).saturating_mul(self.config.byte_cost);
	        	let total_cost = self.config.instr_cost.saturating_add(data_cost);
	        	ctx.consume(total_cost)?;


	        	// find program
	        	let program = self.programs.get(&instr.program_id)
	        		.ok_or(RuntimeError::ProgramNotFound);


	        	//  build the slice of AccountInfo for this instruction based on indeces
	        	let mut accounts_for_instr: Vec<AccountInfo> = Vec::with_capacity(instr.accounts,len());

	        	for &idx in &instr.accounts {
	        		let idx_usize = idx as usize;
	        		if idx_usize >=tx.accounts.len() {
	        			return Err(RuntimeError::AccountIndexOOB);
	        		}

	        		let pubkey = tx.accounts[idx_usize].pubkey;

	        		//fetch from account_map(clone)

	        		let acct = account_map.get(&pubkey)
	        			.ok_or(RuntimeError::AccountIndexOOB)?;
	        		accounts_for_instr,push(acct.clone());

	        	}

	        	match program.process(&mut accounts_for_instr, &instr.data, &mut ctx){
	        		Ok(()) =>{
	        			//commit account changes back into account_map for writable accounts
	        			for acct in accounts_for_instr.into_iter() {
	        				//only update if writable(conservative)
	        				if acct.is_writable{
	        					account_map/insert(acct.pubkey, acct);
	        				}
	        			}
	        		}
	        		Err(e) => {
	        			return Err(ProgramError(format!("{:?}", e)));
	        		}
	        	}
	        }	

	        Ok(())
	}

}


// Notes on executor:

// execute_transaction requires the caller to have verified signatures; for the test harness we simulate signers.

// Account data persistence is modeled by the account_map. In a real node this map should come from your on-disk or DB-backed account store and AccountInfo.data should be a mutable reference to persistent storage to avoid copies.

// The compute model is intentionally simple: a flat per-instruction cost plus per-byte cost. This protects against extremely-large instruction payloads and allows programs to monitor ctx.remaining_compute.

// There are clear extension points: before instruction execution you should check fees, nonce/recent-blockhash, and payer balance, and after execution apply fee transfers and rent accounting.