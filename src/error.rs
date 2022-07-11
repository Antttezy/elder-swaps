use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error, FromPrimitive)]
pub enum SwapError {
    #[error("Invalid ID provided")]
    BadInstructionId,

    #[error("Acoount is not signer")]
    SignerRequired,

    #[error("Acoount is not writeable")]
    WriteableRequired,

    #[error("Acoount is not properly derived")]
    PdaCheckFailed,

    #[error("Provided account address doesnt match with one in config")]
    AddressMismatch,
}

impl PrintProgramError for SwapError {
    fn print<E>(&self) {
        msg!(&self.to_string())
    }
}

impl From<SwapError> for ProgramError {
    fn from(e: SwapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SwapError {
    fn type_of() -> &'static str {
        "SwapError"
    }
}
