use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::error::SwapError;

pub fn assert_signer(acc: &AccountInfo) -> Result<(), ProgramError> {
    match acc.is_signer {
        true => Ok(()),
        false => Err(SwapError::SignerRequired.into()),
    }
}

pub fn assert_writeable(acc: &AccountInfo) -> Result<(), ProgramError> {
    match acc.is_writable {
        true => Ok(()),
        false => Err(SwapError::WriteableRequired.into()),
    }
}

pub fn assert_owned_by(acc: &AccountInfo, expected_owner: &Pubkey) -> Result<(), ProgramError> {
    match acc.owner.eq(expected_owner) {
        true => Ok(()),
        false => Err(ProgramError::IllegalOwner),
    }
}
