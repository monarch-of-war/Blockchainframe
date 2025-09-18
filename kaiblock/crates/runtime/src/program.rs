use crate::types::AccountInfo;
use thiserror::Error;

/// Program trait: implement this for any native program you want to register
/// with the runtime. For WASM-backed programs you will implement a small
/// shim that loads the module and provides the same `process` signature.

#[derive(Error, Debug)]
pubb enum ProgramError {
	#[error("program error: {0}")]
	Custom(String),
}


pub trait Program: Send + Sync {

    /// Process an instruction given a slice of AccountInfo (in the order specified by the instruction)
    /// and the instruction binary data.
    ///
    /// RuntimeContext provides limited host functions and the compute budget.

    fn process(
    	&self,
    	accounts: &mut [AccountInfo],
    	data: &[u8],
    	ctx: &mut crate::executor::RuntimeContext,
    	) ->Result<(), ProgramError>;
}