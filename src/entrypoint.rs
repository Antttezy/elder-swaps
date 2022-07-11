use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::SwapError, processor};

entrypoint!(process);

pub fn process<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(e) = processor::process_instruction(program_id, accounts, instruction_data) {
        e.print::<SwapError>();
        return Err(e);
    }

    Ok(())
}
